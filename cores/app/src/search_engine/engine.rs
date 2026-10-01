//! Search worker: generation, cancellation, chunked streaming (handover §3, §16).
//!
//! - Every query/root change calls [`Engine::bump`]: the generation counter
//!   (`AtomicU64`) advances and every tracked rg/sk process of older generations
//!   is killed and reaped synchronously, so processes never accumulate (AC-114).
//! - Workers re-check their generation between steps and exit when stale; the
//!   UI also drops events of non-current generations (Latest Query Wins, §3.2).
//! - rg output is read on a helper thread and forwarded in chunks. Each chunk is
//!   filtered by one short-lived `sk --filter` run and sent to the UI immediately.

use std::io::{BufRead, BufReader, Read};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::log;

use super::executable::CREATE_NO_WINDOW;
use super::ripgrep::{self, Hit};
use super::skim;
use crate::query_syntax::ParsedQuery;

/// The first chunk is flushed as soon as the available rg output has been drained
/// (first-result latency); later chunks are batched up to `NEXT_FLUSH` / `MAX_CHUNK`.
const NEXT_FLUSH: Duration = Duration::from_millis(30);
const MAX_CHUNK: usize = 5000;
const POLL: Duration = Duration::from_millis(1);

#[derive(Debug, Clone)]
pub struct Request {
    pub generation: u64,
    pub query: ParsedQuery,
    pub root: PathBuf,
    pub rg: PathBuf,
    pub sk: PathBuf,
    /// Time the query change was detected (latency base).
    pub changed_at: Instant,
}

#[derive(Debug)]
pub enum Event {
    Batch { generation: u64, hits: Vec<Hit> },
    Done { generation: u64, total: usize, elapsed: Duration },
    Error { generation: u64, message: String },
}

#[derive(Default)]
struct Procs {
    next_key: u64,
    /// (key, generation, name, process)
    live: Vec<(u64, u64, &'static str, Child)>,
}

#[derive(Clone, Default)]
pub struct Engine {
    current: Arc<AtomicU64>,
    procs: Arc<Mutex<Procs>>,
    /// sk spawned at query-change time for the pending generation (before coalescing ends)
    early: Arc<Mutex<Option<(u64, PreparedSk)>>>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a new generation and kills every tracked process of older generations.
    pub fn bump(&self) -> u64 {
        let new = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        let killed = self.kill_older_than(new);
        log::write(
            "search_engine",
            &format!("generation={new} cancelled_generation={} killed={killed:?}", new - 1),
        );
        new
    }

    pub fn current(&self) -> u64 {
        self.current.load(Ordering::SeqCst)
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.current() == generation
    }

    /// Spawns sk for `generation` right at query change so its startup overlaps with the
    /// 20ms coalescing window. Replaces (and releases) any previous early sk.
    pub fn prepare_early(&self, generation: u64, sk: &Path, expr: &str, changed_at: Instant) {
        let prepared = match prepare_sk(self, generation, sk, expr, changed_at) {
            Ok(p) => p,
            Err(e) => {
                log::write("search_engine", &format!("generation={generation} early sk spawn failed: {e}"));
                None
            }
        };
        let old = self.early.lock().ok().and_then(|mut slot| match prepared {
            Some(p) => slot.replace((generation, p)),
            None => slot.take(),
        });
        discard(self, old.map(|o| o.1));
    }

    /// Takes the early sk if it belongs to `generation`; a stale one is released.
    fn take_early(&self, generation: u64) -> Option<PreparedSk> {
        let taken = self.early.lock().ok()?.take()?;
        if taken.0 == generation {
            Some(taken.1)
        } else {
            discard(self, Some(taken.1));
            None
        }
    }

    /// Releases the early sk (search not started: invalid root, empty query).
    pub fn drop_early(&self) {
        let old = self.early.lock().ok().and_then(|mut slot| slot.take());
        discard(self, old.map(|o| o.1));
    }

    /// Generations of the rg/sk processes currently alive under this engine.
    pub fn live_generations(&self) -> Vec<u64> {
        self.procs.lock().map(|p| p.live.iter().map(|l| l.1).collect()).unwrap_or_default()
    }

    fn kill_older_than(&self, generation: u64) -> Vec<String> {
        let Ok(mut procs) = self.procs.lock() else { return Vec::new() };
        let mut killed = Vec::new();
        let mut keep = Vec::new();
        for (key, gen, name, mut child) in procs.live.drain(..) {
            if gen < generation {
                let _ = child.kill();
                let _ = child.wait();
                killed.push(format!("{name}:{}(gen {gen})", child.id()));
            } else {
                keep.push((key, gen, name, child));
            }
        }
        procs.live = keep;
        killed
    }

    /// Tracks a process. If its generation is already stale it is killed immediately.
    fn track(&self, generation: u64, name: &'static str, mut child: Child) -> Option<u64> {
        let mut procs = self.procs.lock().ok()?;
        if !self.is_current(generation) {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        procs.next_key += 1;
        let key = procs.next_key;
        procs.live.push((key, generation, name, child));
        Some(key)
    }

    /// Removes a tracked process and reaps it. `None` if it was already killed by `bump`.
    fn finish(&self, key: u64) -> Option<std::process::ExitStatus> {
        let mut child = {
            let mut procs = self.procs.lock().ok()?;
            let pos = procs.live.iter().position(|p| p.0 == key)?;
            procs.live.remove(pos).3
        };
        child.wait().ok()
    }

    pub fn spawn<F>(&self, req: Request, sink: F) -> std::thread::JoinHandle<()>
    where
        F: Fn(Event) + Send + 'static,
    {
        let engine = self.clone();
        std::thread::spawn(move || run(engine, req, sink))
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn run<F: Fn(Event)>(engine: Engine, req: Request, sink: F) {
    let gen = req.generation;
    let pattern = skim::prefilter_regex(&req.query.skim);
    let globs = req.query.scope.rg_globs();
    log::write(
        "search_engine",
        &format!(
            "generation={gen} raw={:?} skim={:?} scope={:?} root={} rg_pattern={pattern:?} rg_globs={globs:?}",
            req.query.raw,
            req.query.skim,
            req.query.scope,
            req.root.display()
        ),
    );

    let spawned = Command::new(&req.rg)
        .args(ripgrep::args(&pattern, &globs, &req.root))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    let mut child = match spawned {
        Ok(c) => c,
        Err(e) => {
            log::write("search_engine", &format!("generation={gen} error=rg spawn failed: {e}"));
            sink(Event::Error { generation: gen, message: format!("rg 실행 실패: {e}") });
            return;
        }
    };
    let pid = child.id();
    let stdout = child.stdout.take().expect("rg stdout");
    let mut stderr = child.stderr.take().expect("rg stderr");
    let Some(rg_key) = engine.track(gen, "rg", child) else {
        log::write("search_engine", &format!("cancelled generation={gen} before rg start"));
        return;
    };
    log::write(
        "search_engine",
        &format!("generation={gen} rg_spawn_ms={:.1} pid={pid}", ms(req.changed_at.elapsed())),
    );
    // sk for the first chunk: spawned at query change (Engine::prepare_early) or now,
    // so its startup overlaps with coalescing / rg (first-result latency, AC-103)
    let early = engine.take_early(gen);
    let mut prepared = match early.map_or_else(|| prepare_sk(&engine, gen, &req.sk, &req.query.skim, req.changed_at), |p| Ok(Some(p))) {
        Ok(p) => p,
        Err(e) => {
            log::write("search_engine", &format!("generation={gen} error=sk spawn failed: {e}"));
            engine.finish(rg_key);
            sink(Event::Error { generation: gen, message: format!("sk 실행 실패: {e}") });
            return;
        }
    };

    let err_reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let (tx, rx) = mpsc::channel::<Hit>();
    let reader = std::thread::spawn(move || {
        let mut r = BufReader::with_capacity(64 * 1024, stdout);
        let mut buf = Vec::new();
        let mut malformed = 0usize;
        loop {
            buf.clear();
            match r.read_until(b'\n', &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => match ripgrep::parse_line(&buf) {
                    Some(hit) => {
                        if tx.send(hit).is_err() {
                            break;
                        }
                    }
                    None => malformed += 1,
                },
            }
        }
        malformed
    });

    let mut chunk: Vec<Hit> = Vec::new();
    let mut chunk_started: Option<Instant> = None;
    let mut total = 0usize;
    let mut raw_total = 0usize;
    let mut first_sent = false;
    let mut eof = false;

    loop {
        if !engine.is_current(gen) {
            engine.finish(rg_key); // already killed by bump; reaps if still tracked
            discard(&engine, prepared.take());
            log::write("search_engine", &format!("cancelled generation={gen} worker_exit sent={total}"));
            return;
        }
        let mut next = rx.recv_timeout(POLL);
        loop {
            match next {
                Ok(hit) => {
                    raw_total += 1;
                    // `!file:` (not an rg glob: it would also skip matching directories)
                    if !req.query.scope.excludes_file(&hit.path) {
                        chunk_started.get_or_insert_with(Instant::now);
                        chunk.push(hit);
                    }
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    eof = true;
                    break;
                }
            }
            if chunk.len() >= MAX_CHUNK {
                break;
            }
            // drain what rg has already produced without waiting
            next = match rx.try_recv() {
                Ok(h) => Ok(h),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => Err(RecvTimeoutError::Disconnected),
            };
        }
        let due = !first_sent || chunk_started.is_some_and(|t| t.elapsed() >= NEXT_FLUSH);
        if !chunk.is_empty() && (eof || due || chunk.len() >= MAX_CHUNK) {
            let batch = std::mem::take(&mut chunk);
            chunk_started = None;
            let kept = match filter_chunk(&engine, &req, batch, prepared.take()) {
                Ok(Some(k)) => k,
                Ok(None) => continue, // cancelled while sk ran
                Err(e) => {
                    log::write("search_engine", &format!("generation={gen} error=sk failed: {e}"));
                    engine.finish(rg_key);
                    sink(Event::Error { generation: gen, message: format!("sk 실행 실패: {e}") });
                    return;
                }
            };
            if !engine.is_current(gen) {
                continue;
            }
            if !kept.is_empty() {
                total += kept.len();
                if !first_sent {
                    log::write(
                        "search_engine",
                        &format!("generation={gen} first_result_ms={:.1}", ms(req.changed_at.elapsed())),
                    );
                }
                first_sent = true;
                sink(Event::Batch { generation: gen, hits: kept });
            }
            if !eof && prepared.is_none() {
                // more output follows: start the next chunk's sk now (after the batch was sent,
                // so it stays off the first-result path) to overlap its startup with rg
                prepared = prepare_sk(&engine, gen, &req.sk, &req.query.skim, req.changed_at).unwrap_or(None);
            }
        }
        if eof && chunk.is_empty() {
            break;
        }
    }

    discard(&engine, prepared.take()); // no output at all: release the unused sk
    let status = engine.finish(rg_key);
    let malformed = reader.join().unwrap_or(0);
    let stderr = err_reader.join().unwrap_or_default();
    let code = status.and_then(|s| s.code());
    let elapsed = req.changed_at.elapsed();
    log::write(
        "search_engine",
        &format!(
            "generation={gen} completed_ms={:.1} rg_lines={raw_total} result_count={total} rg_exit={code:?} malformed={malformed}",
            ms(elapsed)
        ),
    );
    if !engine.is_current(gen) {
        return;
    }
    // rg: 0 = matches, 1 = no match, 2 = error (partial results may still exist)
    if code == Some(2) && !stderr.trim().is_empty() {
        log::write("search_engine", &format!("generation={gen} error=rg stderr: {}", stderr.trim()));
        if total == 0 {
            let first = stderr.lines().next().unwrap_or_default().to_string();
            sink(Event::Error { generation: gen, message: first });
            return;
        }
    }
    sink(Event::Done { generation: gen, total, elapsed });
}

/// An sk process spawned ahead of its chunk (startup overlaps with rg).
struct PreparedSk {
    key: u64,
    pid: u32,
    stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
    spawned: Instant,
}

/// Spawns and tracks sk for the next chunk. `Ok(None)` if the expression is empty or cancelled.
fn prepare_sk(engine: &Engine, generation: u64, sk: &Path, expr: &str, changed_at: Instant) -> std::io::Result<Option<PreparedSk>> {
    if expr.trim().is_empty() {
        return Ok(None);
    }
    let spawned = Instant::now();
    let mut child = skim::spawn_filter(sk, expr)?;
    let pid = child.id();
    let stdin = child.stdin.take().expect("sk stdin");
    let stdout = child.stdout.take().expect("sk stdout");
    let prepared = engine.track(generation, "sk", child).map(|key| PreparedSk { key, pid, stdin, stdout, spawned });
    if prepared.is_some() {
        log::write("search_engine", &format!("generation={generation} sk_spawn_ms={:.1} pid={pid}", ms(changed_at.elapsed())));
    }
    Ok(prepared)
}

/// `Ok(None)` when the generation was cancelled while sk was running.
/// `prepared` is a pre-spawned sk for this chunk; otherwise one is spawned now.
fn filter_chunk(
    engine: &Engine,
    req: &Request,
    batch: Vec<Hit>,
    prepared: Option<PreparedSk>,
) -> std::io::Result<Option<Vec<Hit>>> {
    if req.query.skim.trim().is_empty() {
        return Ok(Some(batch));
    }
    let t = Instant::now();
    let reused = prepared.is_some();
    let sk = match prepared {
        Some(p) => p,
        None => match prepare_sk(engine, req.generation, &req.sk, &req.query.skim, req.changed_at)? {
            Some(p) => p,
            None => return Ok(None),
        },
    };
    let records: Vec<(usize, &str)> = batch.iter().enumerate().map(|(i, h)| (i, h.text.as_str())).collect();
    let ids = skim::exchange(sk.stdin, sk.stdout, &records)?;
    if engine.finish(sk.key).is_none() {
        return Ok(None); // killed by bump
    }
    log::write(
        "search_engine",
        &format!(
            "generation={} sk_run_ms={:.1} sk_age_ms={:.1} prespawned={reused} pid={} in={} out={} payload=id\tcontent nth=2..",
            req.generation,
            ms(t.elapsed()),
            ms(sk.spawned.elapsed()),
            sk.pid,
            batch.len(),
            ids.len()
        ),
    );
    let mut keep = vec![false; batch.len()];
    for id in ids {
        if let Some(k) = keep.get_mut(id) {
            *k = true;
        }
    }
    Ok(Some(batch.into_iter().zip(keep).filter_map(|(h, k)| k.then_some(h)).collect()))
}

/// Releases an unused pre-spawned sk: closing stdin makes it exit, then it is reaped.
fn discard(engine: &Engine, prepared: Option<PreparedSk>) {
    if let Some(p) = prepared {
        drop(p.stdin);
        drop(p.stdout);
        engine.finish(p.key);
    }
}
