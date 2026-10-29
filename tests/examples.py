"""Compile compact examples as C++17 and shared C examples as C11."""
import argparse
import concurrent.futures
import json
import os
import pathlib
import subprocess
import tempfile


def check(entry, compiler, language, standard, run):
    with tempfile.TemporaryDirectory(prefix="cpp-docs-example-") as temporary:
        command = [compiler, f"-std={standard}", "-x", language]
        executable = pathlib.Path(temporary) / ("example.exe" if os.name == "nt" else "example")
        if run:
            command.extend(["-o", str(executable)])
        else:
            command.append("-fsyntax-only")
        result = subprocess.run(command + ["-"], input=entry["example"], text=True, capture_output=True, timeout=60)
        assert result.returncode == 0, f"{entry['symbol']} ({standard}): {result.stderr}"
        if run:
            result = subprocess.run([str(executable)], input="42\n", text=True, capture_output=True, cwd=temporary, timeout=10)
            assert result.returncode == 0, f"{entry['symbol']} ({standard}): {result.stdout}\n{result.stderr}"
    return f"PASS {standard} example {entry['symbol']}"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("compiler", nargs="?", default="clang++")
    parser.add_argument("--c-compiler", default="clang")
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    root = pathlib.Path(__file__).parent.parent
    entries = json.loads((root / "data/compact-docs.json").read_text(encoding="utf-8"))
    sources = {source["symbol"]: source for source in json.loads((root / "data/compact-docs-sources.json").read_text(encoding="utf-8"))}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
        tasks = []
        for entry in entries:
            tasks.append(executor.submit(check, entry, args.compiler, "c++", "c++17", args.run))
            if sources[entry["symbol"]]["language"] == "c":
                tasks.append(executor.submit(check, entry, args.c_compiler, "c", "c11", args.run))
        for task in tasks:
            print(task.result(), flush=True)
