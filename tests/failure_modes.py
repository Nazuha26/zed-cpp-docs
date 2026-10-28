"""Dependency and offline fallback checks against the real proxy."""
import json
import os
import pathlib
import subprocess
import sys
import tempfile

from integration import Client, position


def run(binary, clangd):
    missing = subprocess.run([binary, "--clangd", "zed-cpp-docs-intentionally-missing-clangd"], capture_output=True, timeout=10)
    assert missing.returncode != 0 and not missing.stdout and b"cannot start clangd" in missing.stderr
    print("PASS missing clangd gives an actionable error without corrupting stdout")
    with tempfile.TemporaryDirectory(prefix="cpp-docs-bootstrap-") as directory:
        root = pathlib.Path(directory)
        index = root / "index.json"
        fixtures = json.loads((pathlib.Path(__file__).parent.parent / "fixtures/docs.json").read_text())
        original = json.dumps({"schema": 1, "entries": fixtures}).encode()
        index.write_bytes(original)
        environment = os.environ.copy()
        environment["PATH"] = ""
        ready = subprocess.run([
            binary, "--clangd", "intentionally-missing-clangd", "--data-dir", directory,
            "--ensure-cppreference",
        ], env=environment, capture_output=True, timeout=10)
        assert ready.returncode == 0 and not ready.stdout, ready.stderr
        assert index.read_bytes() == original and not (root / "downloads").exists()
        print("PASS automatic setup reuses an installed index without clangd, curl or network")
        archive = root / "corrupt.tar.xz"
        archive.write_bytes(b"not the official cppreference archive")
        invalid = subprocess.run([
            binary, "--data-dir", directory, "--import-cppreference", str(archive),
        ], capture_output=True, timeout=10)
        assert invalid.returncode != 0 and not invalid.stdout and b"SHA-256 mismatch" in invalid.stderr
        assert index.read_bytes() == original and not (root / "install.lock").exists()
        print("PASS corrupted archive rejected without replacing the index")
    with tempfile.TemporaryDirectory(prefix="cpp-docs-empty-") as directory:
        client = Client([binary, "--data-dir", directory, "--clangd", clangd, "--", "--log=error"])
        try:
            root = pathlib.Path(__file__).parent.resolve()
            uri = (root / "samples/standard.cpp").as_uri()
            text = (root / "samples/standard.cpp").read_text()
            client.request("initialize", {"processId": None, "rootUri": root.as_uri(), "capabilities": {"textDocument": {"hover": {"contentFormat": ["markdown"]}}}}, 1)
            client.send("initialized", {})
            client.send("textDocument/didOpen", {"textDocument": {"uri": uri, "version": 1, "languageId": "cpp", "text": text}})
            params = {"textDocument": {"uri": uri}, "position": position(text, "getline(input")}
            hover = client.request("textDocument/hover", params, 2)
            assert "getline" in json.dumps(hover) and "Source:" not in json.dumps(hover)
            navigation = client.request("textDocument/definition", params, 3)
            assert navigation.get("result")
            assert any(message["method"] == "window/showMessage" for message in client.notifications)
            print("PASS missing documentation warns the user and preserves clangd hover/navigation")
        finally:
            client.close()


if __name__ == "__main__":
    run(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "clangd")
