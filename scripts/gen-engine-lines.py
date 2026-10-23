#!/usr/bin/env python3
"""Collects the engine's explanation lines for all example characters (via `gurps calc`)
into ui/tests/engine-lines.json, so the ru translation map is tested against real output.
Run after `cargo build -p gurps-cli`."""
import json, subprocess
from pathlib import Path

R = Path(__file__).resolve().parent.parent
lines = set()


def walk(x, k=None):
    if isinstance(x, dict):
        for kk, v in x.items():
            walk(v, kk)
    elif isinstance(x, list):
        for v in x:
            if k == "explain" and isinstance(v, str):
                lines.add(v)
            else:
                walk(v, k)
    elif isinstance(x, str) and k in ("base_reason", "reason", "message"):
        lines.add(x)


for f in sorted([*R.glob("examples/test-chars/*.json"), *R.glob("examples/*.json")]):
    out = subprocess.run([R / "target/debug/gurps", "calc", f], capture_output=True, text=True)
    if out.returncode == 0:
        walk(json.loads(out.stdout))
(R / "ui/tests/engine-lines.json").write_text(json.dumps(sorted(lines), ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
print(len(lines))
