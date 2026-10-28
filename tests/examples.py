"""Syntax-check the complete compact examples with a C++ compiler."""
import json
import pathlib
import subprocess
import sys


for entry in json.loads((pathlib.Path(__file__).parent.parent / "data/compact-docs.json").read_text()):
    result = subprocess.run([sys.argv[1] if len(sys.argv) > 1 else "clang++", "-std=c++17", "-x", "c++", "-fsyntax-only", "-"], input=entry["example"], text=True, capture_output=True)
    assert result.returncode == 0, f"{entry['symbol']}: {result.stderr}"
    print(f"PASS example {entry['symbol']}")
