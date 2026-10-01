"""Generate the path constants region of skim_search/__init__.py from cores/common/paths.ini.

    python -m skim_search.gen_paths           rewrite the region
    python -m skim_search.gen_paths --check   exit 1 if the committed region differs (CI, tests)

The generated code is the fastest form measured for this package (build_env/paths-ssot):
string constants in the package __init__ (no extra module import, no file read, no pathlib).
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

INIT = Path(__file__).with_name("__init__.py")
BEGIN = "# --- GENERATED from cores/common/paths.ini by `python -m skim_search.gen_paths`; do not edit ---\n"
END = "# --- END GENERATED ---\n"
MAX_KEYS = 200


def parse(text: str) -> list[tuple[str, str, str]]:
    entries, section, seen = [], "", set()
    for n, raw in enumerate(text.splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            continue
        key, sep, value = line.partition("=")
        key, value = key.strip(), value.strip()
        if not sep or not section or not key.replace("_", "").isalnum() or not key.isupper() or key in seen:
            raise ValueError(f"paths.ini:{n}: invalid or duplicate entry {line!r}")
        if not value or "\\" in value or ":" in value or value.startswith("/"):
            raise ValueError(f"paths.ini:{n}: {key} must be a relative '/' path")
        seen.add(key)
        entries.append((section, key, value))
    return entries


def render(entries: list[tuple[str, str, str]], depth: int) -> str:
    repo = [(k, v) for s, k, v in entries if s == "repo"]
    third = [(k, v) for s, k, v in entries if s == "third_party"]
    third_root = dict(third).pop("THIRD_PARTY")
    lines = [
        BEGIN,
        f"_F = __file__.replace(\"\\\\\", \"/\")\n",
        f"ROOT = _F.rsplit(\"/\", {depth})[0]  # repository root ({depth} levels above this file)\n",
    ]
    lines += [f"{k} = ROOT + {('/' + v)!r}\n" for k, v in repo]
    lines.append("REL = {" + ", ".join(f"{k!r}: {v!r}" for _, k, v in entries) + "}\n")
    lines.append(f"_THIRD_PARTY_KEYS = {tuple(k for k, _ in third)!r}\n")
    lines += [
        "\n",
        "def __getattr__(name):\n",
        "    # [third_party] values: the 3rd_party directory is found lazily (filesystem walk) on first use\n",
        "    if name not in _THIRD_PARTY_KEYS:\n",
        "        raise AttributeError(name)\n",
        "    import os\n",
        "    d = ROOT\n",
        "    while True:\n",
        f"        tp = d + {('/' + third_root)!r}\n",
        f"        if os.path.isfile(tp + {('/' + dict(third)['RG'])!r}):\n",
        "            break\n",
        "        parent = d.rpartition(\"/\")[0]\n",
        "        if not parent or parent == d:\n",
        f"            raise AttributeError(f\"{{name}}: no {third_root} above {{ROOT}}\")\n",
        "        d = parent\n",
        "    values = {k: (tp if k == 'THIRD_PARTY' else tp + '/' + REL[k]) for k in _THIRD_PARTY_KEYS}\n",
        "    globals().update(values)\n",
        "    return values[name]\n",
        END,
    ]
    return "".join(lines)


def depth_of(entries) -> int:
    project = dict((k, v) for _, k, v in entries)["PYTHON_PROJECT"]
    return len(project.split("/")) + 3  # <project>/src/skim_search/__init__.py


def generated(ini: Path) -> str:
    entries = parse(ini.read_text(encoding="utf-8"))
    if len(entries) > MAX_KEYS:
        print(f"warning: paths.ini has {len(entries)} keys (> {MAX_KEYS}): revisit the structure", file=sys.stderr)
    return render(entries, depth_of(entries))


def splice(current: str, region: str) -> str:
    if BEGIN in current and END in current:
        head, _, rest = current.partition(BEGIN)
        _, _, tail = rest.partition(END)
        return head + region + tail
    return region + current


def find_ini() -> Path:
    for d in INIT.parents:
        candidate = d / "cores" / "common" / "paths.ini"
        if candidate.is_file():
            return candidate
    raise SystemExit("cores/common/paths.ini not found above the package")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    current = INIT.read_text(encoding="utf-8") if INIT.exists() else ""
    wanted = splice(current, generated(find_ini()))
    if args.check:
        if wanted != current:
            print("skim_search/__init__.py is out of date: run python -m skim_search.gen_paths", file=sys.stderr)
            return 1
        print("paths up to date")
        return 0
    if wanted != current:
        INIT.write_text(wanted, encoding="utf-8", newline="\n")
        print(f"wrote {INIT}")
    else:
        print("paths up to date")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
