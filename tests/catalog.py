"""Verify every curated symbol against the imported index and real clangd hover."""
import json
import pathlib
import re
import sys
import tempfile

from integration import Client, position


def hover_source(entry):
    symbol = entry["symbol"]
    text = entry["example"]
    if "::operator[]" in symbol:
        match = re.search(r"\b(values|text)\[(\d+)\]", text)
        assert match, symbol
        text = text[:match.start()] + f"{match[1]}.operator[]({match[2]})" + text[match.end():]
        return text, "operator[]"
    if symbol.endswith("::npos"):
        return text, "npos"
    if symbol in ["std::pair::first", "std::pair::second"]:
        name = symbol.rsplit("::", 1)[1]
        return text, name + " =="
    if symbol.startswith(("std::vector::", "std::basic_string::", "std::map::")):
        name = symbol.rsplit("::", 1)[1]
        return text, name + "("
    needle = symbol
    lines = text.splitlines(keepends=True)
    for line in lines:
        if not line.startswith("#") and re.search(r"\b" + re.escape(needle) + r"\b", line):
            return text, needle
    raise AssertionError(f"No hover target in example: {symbol}")


def run(binary, clangd, data):
    root = pathlib.Path(__file__).resolve().parent.parent
    entries = json.loads((root / "data/compact-docs.json").read_text(encoding="utf-8"))
    sources = json.loads((root / "data/compact-docs-sources.json").read_text(encoding="utf-8"))
    assert len(entries) == len(sources) == 100
    assert len({entry["symbol"] for entry in entries}) == 100
    source_by_symbol = {source["symbol"]: source for source in sources}
    assert {entry["symbol"] for entry in entries} == set(source_by_symbol)
    index = json.loads((pathlib.Path(data) / "index.json").read_text(encoding="utf-8"))["entries"]
    for entry in entries:
        source = source_by_symbol[entry["symbol"]]
        matches = [item for item in index if item["symbol"] == entry["symbol"]]
        assert matches, entry["symbol"]
        assert source["cppreference"] in [item["source_url"] for item in matches], entry["symbol"]
        assert source["standard"].startswith(("https://eel.is/c++draft/", "https://port70.net/~nsz/c/c11/n1570.html#"))
        for field in ["summary", "signature", "example"]:
            assert entry[field].strip(), (entry["symbol"], field)
    print("PASS 100 unique descriptions with matching cppreference pages and standard references", flush=True)
    failures = []
    with tempfile.TemporaryDirectory(prefix="cpp-docs-catalog-") as temporary:
        descriptions = str(pathlib.Path(temporary) / "no-user-descriptions.json")
        for language, standard, extension in [("cpp", "c++17", "cpp"), ("c", "c11", "c")]:
            client = Client([binary, "--clangd", clangd, "--data-dir", data, "--descriptions", descriptions, "--", "--log=error", "--background-index=false"])
            try:
                client.request("initialize", {
                    "processId": None,
                    "rootUri": root.as_uri(),
                    "initializationOptions": {"fallbackFlags": [f"-std={standard}"]},
                    "capabilities": {"textDocument": {"hover": {"contentFormat": ["markdown"]}}},
                }, 1)
                client.send("initialized", {})
                request_id = 2
                for number, entry in enumerate(entries):
                    if source_by_symbol[entry["symbol"]]["language"] != language:
                        continue
                    text, needle = hover_source(entry)
                    # A repeated spelling in a header name must not select the include directive.
                    start = text.index("int main(void)")
                    target = text.index(needle, start)
                    if needle.startswith("std::"):
                        target += len("std::")
                    uri = (root / "tests" / "samples" / f"catalog-{number}.{extension}").as_uri()
                    client.send("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": language, "version": 1, "text": text}})
                    params = {"textDocument": {"uri": uri}, "position": position(text, text[target:])}
                    result = client.request("textDocument/hover", params, request_id)
                    request_id += 1
                    markdown = (result.get("result") or {}).get("contents", {})
                    markdown = markdown.get("value", "") if isinstance(markdown, dict) else str(markdown)
                    if entry["summary"] not in markdown or "Includes compact descriptions" not in markdown:
                        symbol = client.request("textDocument/symbolInfo", params, request_id)
                        declaration = client.request("textDocument/declaration", params, request_id + 1)
                        request_id += 2
                        failures.append(f"{entry['symbol']}: {result}\nSymbol: {symbol}\nDeclaration: {declaration}")
                        print("FAIL hover", entry["symbol"], flush=True)
                    else:
                        assert "[Open full local documentation]" in markdown, entry["symbol"]
                        print("PASS curated hover", entry["symbol"], flush=True)
                    client.send("textDocument/didClose", {"textDocument": {"uri": uri}})
            finally:
                client.close()
    assert not failures, "\n\n".join(failures)


if __name__ == "__main__":
    run(*sys.argv[1:4])
