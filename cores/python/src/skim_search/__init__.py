# --- GENERATED from cores/common/paths.ini by `python -m skim_search.gen_paths`; do not edit ---
_F = __file__.replace("\\", "/")
ROOT = _F.rsplit("/", 5)[0]  # repository root (5 levels above this file)
PATHS_INI = ROOT + '/cores/common/paths.ini'
CARGO_WORKSPACE = ROOT + '/cores/rust'
RUST_LOCK = ROOT + '/cores/rust/Cargo.lock'
PYTHON_PROJECT = ROOT + '/cores/python'
SAMPLE_TREE = ROOT + '/cores/rust/tests/fixtures/sample/tree'
SCRIPTS = ROOT + '/scripts'
ONE_SHOT_SCRIPTS = ROOT + '/scripts/one_shot'
CONFIGS = ROOT + '/configs'
ONE_SHOT_CONFIG = ROOT + '/configs/one-shot.json'
SECURITY_EXCEPTIONS = ROOT + '/configs/security-exceptions.json'
ISSUES = ROOT + '/issues'
PRIORITY = ROOT + '/issues/priority.md'
TARGET = ROOT + '/target'
ONE_SHOT_TARGET = ROOT + '/target/one-shot'
EXE_DEBUG = ROOT + '/target/debug/skim-search.exe'
EXE_RELEASE = ROOT + '/target/release/skim-search.exe'
EVIDENCE = ROOT + '/ref/actual'
LOGS = ROOT + '/ref/actual/logs'
APP_LOG = ROOT + '/ref/actual/logs/skim-search.log'
ONE_SHOT_LOGS = ROOT + '/ref/actual/logs/one-shot'
SECURITY_LOGS = ROOT + '/ref/actual/logs/security'
SHOTS = ROOT + '/ref/actual/screenshot/frames'
REL = {'PATHS_INI': 'cores/common/paths.ini', 'CARGO_WORKSPACE': 'cores/rust', 'RUST_LOCK': 'cores/rust/Cargo.lock', 'PYTHON_PROJECT': 'cores/python', 'SAMPLE_TREE': 'cores/rust/tests/fixtures/sample/tree', 'SCRIPTS': 'scripts', 'ONE_SHOT_SCRIPTS': 'scripts/one_shot', 'CONFIGS': 'configs', 'ONE_SHOT_CONFIG': 'configs/one-shot.json', 'SECURITY_EXCEPTIONS': 'configs/security-exceptions.json', 'ISSUES': 'issues', 'PRIORITY': 'issues/priority.md', 'TARGET': 'target', 'ONE_SHOT_TARGET': 'target/one-shot', 'EXE_DEBUG': 'target/debug/skim-search.exe', 'EXE_RELEASE': 'target/release/skim-search.exe', 'EVIDENCE': 'ref/actual', 'LOGS': 'ref/actual/logs', 'APP_LOG': 'ref/actual/logs/skim-search.log', 'ONE_SHOT_LOGS': 'ref/actual/logs/one-shot', 'SECURITY_LOGS': 'ref/actual/logs/security', 'SHOTS': 'ref/actual/screenshot/frames', 'THIRD_PARTY': '3rd_party', 'RG_DIR': 'ripgrep', 'SK_DIR': 'skim', 'RG': 'ripgrep/rg.exe', 'SK': 'skim/sk.exe', 'UV': 'pk_system/uv.exe', 'GITLEAKS': 'security/gitleaks.exe', 'RELEASES': 'skim-search', 'VERSIONS': 'skim-search/versions.json'}
_THIRD_PARTY_KEYS = ('THIRD_PARTY', 'RG_DIR', 'SK_DIR', 'RG', 'SK', 'UV', 'GITLEAKS', 'RELEASES', 'VERSIONS')

def __getattr__(name):
    # [third_party] values: the 3rd_party directory is found lazily (filesystem walk) on first use
    if name not in _THIRD_PARTY_KEYS:
        raise AttributeError(name)
    import os
    d = ROOT
    while True:
        tp = d + '/3rd_party'
        if os.path.isfile(tp + '/ripgrep/rg.exe'):
            break
        parent = d.rpartition("/")[0]
        if not parent or parent == d:
            raise AttributeError(f"{name}: no 3rd_party above {ROOT}")
        d = parent
    values = {k: (tp if k == 'THIRD_PARTY' else tp + '/' + REL[k]) for k in _THIRD_PARTY_KEYS}
    globals().update(values)
    return values[name]
# --- END GENERATED ---
