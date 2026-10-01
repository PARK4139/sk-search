//! Toast stack (handover §23): bottom-right, newest at the bottom, max 3,
//! auto dismiss, info/success/warning/error.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use common::log;
use slint::{Model, VecModel};

use crate::ToastData;

pub const MAX_TOASTS: usize = 3;

pub fn duration(kind: &str) -> Duration {
    match kind {
        "warning" | "error" => Duration::from_millis(6000),
        _ => Duration::from_millis(3000),
    }
}

thread_local! {
    static NEXT_ID: Cell<i32> = const { Cell::new(1) };
}

/// Pushes a toast; drops the oldest beyond `MAX_TOASTS`; schedules auto removal.
pub fn push(model: &Rc<VecModel<ToastData>>, kind: &str, title: &str, body: &str) {
    let id = NEXT_ID.with(|n| {
        let id = n.get();
        n.set(id + 1);
        id
    });
    model.push(ToastData { id, kind: kind.into(), title: title.into(), body: body.into() });
    while model.row_count() > MAX_TOASTS {
        let dropped = model.remove(0);
        log::write("toast", &format!("drop oldest id={} title={}", dropped.id, dropped.title));
    }
    log::write("toast", &format!("push id={id} kind={kind} title={title} body={body} count={}", model.row_count()));
    let weak = Rc::downgrade(model);
    slint::Timer::single_shot(duration(kind), move || {
        if let Some(model) = weak.upgrade() {
            if let Some(pos) = (0..model.row_count()).find(|&i| model.row_data(i).is_some_and(|t| t.id == id)) {
                model.remove(pos);
                log::write("toast", &format!("auto dismiss id={id} count={}", model.row_count()));
            }
        }
    });
}
