"""Real-clangd integration checks. Uses only the Python standard library."""
import json
import pathlib
import queue
import subprocess
import sys
import threading
import time
import urllib.request


class Client:
    def __init__(self, command):
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.messages = queue.Queue()
        self.notifications = []
        self.stderr = []
        threading.Thread(target=self.read, daemon=True).start()
        threading.Thread(target=lambda: self.stderr.extend(self.process.stderr.read().decode(errors="replace").splitlines()), daemon=True).start()

    def read(self):
        try:
            while True:
                headers = {}
                while True:
                    line = self.process.stdout.readline()
                    if not line:
                        self.messages.put(RuntimeError("Proxy output closed"))
                        return
                    if line == b"\r\n":
                        break
                    key, value = line.decode().split(":", 1)
                    headers[key.lower()] = value.strip()
                self.messages.put(json.loads(self.process.stdout.read(int(headers["content-length"]))))
        except Exception as error:
            self.messages.put(error)

    def send(self, method, params=None, request_id=None):
        value = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            value["params"] = params
        if request_id is not None:
            value["id"] = request_id
        self.write(value)

    def write(self, value):
        body = json.dumps(value, ensure_ascii=False).encode()
        self.process.stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)
        self.process.stdin.flush()

    def response(self, request_id, timeout=30):
        deadline = time.monotonic() + timeout
        while True:
            value = self.messages.get(timeout=max(0.01, deadline - time.monotonic()))
            if isinstance(value, Exception):
                raise value
            if "method" in value:
                if "id" in value:
                    result = [] if value["method"] == "workspace/configuration" else None
                    self.write({"jsonrpc": "2.0", "id": value["id"], "result": result})
                else:
                    self.notifications.append(value)
                continue
            assert value.get("id") == request_id, f"Unexpected/internal reply: {value}"
            return value

    def request(self, method, params, request_id):
        self.send(method, params, request_id)
        return self.response(request_id)

    def close(self):
        try:
            self.request("shutdown", None, "shutdown")
            self.send("exit")
            self.process.wait(timeout=5)
            assert self.process.returncode == 0, self.stderr
        finally:
            if self.process.poll() is None:
                self.process.kill()
            self.process.wait()


def position(text, needle):
    offset = text.index(needle)
    prefix = text[:offset]
    line = prefix.count("\n")
    character = len(prefix.rsplit("\n", 1)[-1].encode("utf-16-le")) // 2
    return {"line": line, "character": character}


def run(binary, real=False, clangd=None, data_dir=None):
    root = pathlib.Path(__file__).resolve().parent
    command = [binary]
    if not real:
        command.append("--fixtures")
    if clangd:
        command.extend(["--clangd", clangd])
    if data_dir:
        command.extend(["--data-dir", data_dir])
    client = Client(command + ["--", "--log=error", "--background-index=false"])
    try:
        initialized = client.request("initialize", {
            "processId": None, "rootUri": root.as_uri(),
            "initializationOptions": {"fallbackFlags": ["-std=c++20"]},
            "capabilities": {"textDocument": {"hover": {"contentFormat": ["markdown"]}}, "general": {"positionEncodings": ["utf-16"]}},
        }, "zed-cpp-docs:1")
        assert initialized["result"]["capabilities"]["completionProvider"]
        client.send("initialized", {})
        sources = [("standard.cpp", "cpp", [("getline(input", "std::getline"), ("push_back(42)", "std::vector::push_back"), ('printf("%s', "printf")])]
        if real:
            sources.append(("types.cpp", "cpp", [
                ("string text", "std::string"),
                ("istringstream input", "std::istringstream"),
                ("vector<int>", "std::vector"),
                ("map<int", "std::map"),
                ("pair<int", "std::pair"),
                ("basic_string<char>", "std::basic_string"),
                ("size_t count", "std::size_t"),
                ("size_type length", "std::basic_string::size_type"),
                ("value_type character", "std::basic_string::value_type"),
                ("size()", "std::basic_string::size"),
                ("operator[](0)", "std::basic_string::operator[]"),
                ("npos;", "std::basic_string::npos"),
                ("iostate state", "std::ios_base::iostate"),
                ("goodbit;", "std::ios_base::goodbit"),
                ("exception exception", "std::exception"),
                ("EOF;", "EOF"),
                ("errno;", "errno"),
                ("first;", "std::pair::first"),
                ("ostream* output", "std::ostream"),
                ("cout;", "std::cout"),
                ("float_round_style rounding", "std::float_round_style"),
                ("round_to_nearest;", "std::round_to_nearest"),
                ("errc condition", "std::errc"),
                ("invalid_argument;", "std::errc::invalid_argument"),
                ("max_digits10;", "std::numeric_limits::max_digits10"),
                ("is_const_v<const", "std::is_const_v"),
                ("remove_const_t<const", "std::remove_const_t"),
            ]))
            sources.append(("concepts.cpp", "cpp", [("same_as<int", "std::same_as")]))
        request_id = 9007199254740993
        for name, language, checks in sources:
            path = root / "samples" / name
            text = path.read_text()
            uri = path.as_uri()
            client.send("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": language, "version": 1, "text": text}})
            for needle, expected in checks:
                params = {"textDocument": {"uri": uri}, "position": position(text, needle)}
                result = client.request("textDocument/hover", params, request_id)
                request_id += 1
                contents = result.get("result", {}).get("contents", {})
                markdown = contents.get("value", "") if isinstance(contents, dict) else str(contents)
                if ("Source:" if real else "Prototype fixture") not in markdown:
                    symbol = client.request("textDocument/symbolInfo", params, request_id)
                    request_id += 1
                    declaration = client.request("textDocument/declaration", params, request_id)
                    request_id += 1
                    raise AssertionError(f"No documentation for {expected}: {result}\nSymbol: {symbol}\nDeclaration: {declaration}")
                assert expected in markdown
                if real:
                    assert "Source: cppreference" in markdown
                    assert "Alternative:" not in markdown
                print(f"PASS hover {expected}")
                if "http://127.0.0.1:" in markdown:
                    url = markdown.split("[Open full local documentation](", 1)[1].split(")", 1)[0]
                    with urllib.request.urlopen(url, timeout=3) as response:
                        page = response.read()
                        assert len(page) > 100 and (real or b"local prototype fixture" in page)
                    print("PASS local browser page")
                navigation = client.request("textDocument/definition", params, request_id)
                request_id += 1
                assert navigation.get("result"), f"Navigation lost for {expected}"
            if name == "standard.cpp":
                params = {"textDocument": {"uri": uri}, "position": position(text, "printf(0)")}
                own = client.request("textDocument/hover", params, request_id)
                request_id += 1
                assert "Prototype fixture" not in json.dumps(own) and "Source:" not in json.dumps(own)
                assert "printf" in json.dumps(own)
                print("PASS user-defined printf preserved")
                completion = client.request("textDocument/completion", {"textDocument": {"uri": uri}, "position": position(text, "values.push_back") | {"character": 11}}, request_id)
                request_id += 1
                assert completion.get("result"), "Completion lost"
                print("PASS completion and navigation")
            if name == "types.cpp":
                for needle in ["string local", "LocalString ownAlias"]:
                    own = client.request("textDocument/hover", {"textDocument": {"uri": uri}, "position": position(text, needle)}, request_id)
                    request_id += 1
                    assert "Source:" not in json.dumps(own), "User type was replaced with standard documentation"
                print("PASS user-defined string and type alias preserved")
            client.send("textDocument/didClose", {"textDocument": {"uri": uri}})
        assert any(item["method"] == "textDocument/publishDiagnostics" for item in client.notifications), "Diagnostics not forwarded"
        print("PASS diagnostics and large/string client IDs")
    finally:
        client.close()


if __name__ == "__main__":
    clangd = sys.argv[sys.argv.index("--clangd") + 1] if "--clangd" in sys.argv else None
    data_dir = sys.argv[sys.argv.index("--data-dir") + 1] if "--data-dir" in sys.argv else None
    run(sys.argv[1], "--real" in sys.argv, clangd, data_dir)
