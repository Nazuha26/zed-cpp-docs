"""Verify every curated symbol against the imported index and real clangd hover."""
import json
import argparse
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
        return text, "operator[]", 0
    if symbol in ["std::pair::first", "std::pair::second"]:
        name = symbol.rsplit("::", 1)[1]
        return text, name + " ==", 0
    body = text[text.index("int main(void)"):]
    if re.search(r"\b" + re.escape(symbol) + r"\b", body):
        offset = symbol.rfind("::") + 2 if "::" in symbol else 0
        return text, symbol, offset
    name = symbol.rsplit("::", 1)[-1]
    member = re.search(r"(?:\.|->)" + re.escape(name) + r"\b", body)
    if member:
        return text, member[0], len(member[0]) - len(name)
    if re.search(r"\b" + re.escape(name) + r"\b", body):
        return text, name, 0
    raise AssertionError(f"No hover target in example: {symbol}")


def run(binary, clangd, data, selected=None):
    root = pathlib.Path(__file__).resolve().parent.parent
    entries = json.loads((root / "data/compact-docs.json").read_text(encoding="utf-8"))
    sources = json.loads((root / "data/compact-docs-sources.json").read_text(encoding="utf-8"))
    assert entries and len(entries) == len(sources)
    assert len({entry["symbol"] for entry in entries}) == len(entries)
    source_by_symbol = {source["symbol"]: source for source in sources}
    assert len(source_by_symbol) == len(sources)
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
    print(f"PASS {len(entries)} unique descriptions with matching cppreference pages and standard references", flush=True)
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
                    if language == "c" and source_by_symbol[entry["symbol"]]["language"] != "c":
                        continue
                    if selected and entry["symbol"] not in selected:
                        continue
                    text, needle, offset = hover_source(entry)
                    # A repeated spelling in a header name must not select the include directive.
                    start = text.index("int main(void)")
                    boundary = r"(?<!\w)" if needle[0].isalnum() or needle[0] == "_" else ""
                    match = re.search(boundary + re.escape(needle) + r"(?!\w)", text[start:])
                    assert match, (entry["symbol"], needle)
                    target = start + match.start() + offset
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
                        failures.append(f"{entry['symbol']} ({standard}): {result}\nSymbol: {symbol}\nDeclaration: {declaration}")
                        print("FAIL hover", entry["symbol"], standard, flush=True)
                    else:
                        assert markdown.startswith("**"), (entry["symbol"], "Missing bold symbol kind")
                        assert "[Open full local documentation]" in markdown, entry["symbol"]
                        print("PASS curated hover", entry["symbol"], standard, flush=True)
                    client.send("textDocument/didClose", {"textDocument": {"uri": uri}})
            finally:
                client.close()
    assert not failures, "\n\n".join(failures)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary")
    parser.add_argument("clangd")
    parser.add_argument("data")
    parser.add_argument("--symbols", nargs="+")
    args = parser.parse_args()
    run(args.binary, args.clangd, args.data, set(args.symbols or []))
