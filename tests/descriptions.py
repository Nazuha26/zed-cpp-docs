"""Check description precedence against real clangd and the installed HTML index."""
import json
import pathlib
import sys
import tempfile

from integration import Client, position


def check(binary, clangd, data, descriptions, custom):
    root = pathlib.Path(__file__).resolve().parent
    client = Client([
        binary, "--clangd", clangd, "--data-dir", data,
        "--descriptions", str(descriptions), "--", "--log=error", "--background-index=false",
    ])
    try:
        client.request("initialize", {
            "processId": None,
            "rootUri": root.as_uri(),
            "initializationOptions": {"fallbackFlags": ["-std=c++20"]},
            "capabilities": {"textDocument": {"hover": {"contentFormat": ["markdown"]}}},
        }, 1)
        client.send("initialized", {})
        sources = {}
        for name in ["standard.cpp", "types.cpp"]:
            path = root / "samples" / name
            text = path.read_text()
            sources[name] = (path.as_uri(), text)
            client.send("textDocument/didOpen", {
                "textDocument": {"uri": path.as_uri(), "languageId": "cpp", "version": 1, "text": text},
            })

        request_id = 2

        def hover(name, needle):
            nonlocal request_id
            uri, text = sources[name]
            result = client.request("textDocument/hover", {
                "textDocument": {"uri": uri}, "position": position(text, needle),
            }, request_id)
            request_id += 1
            return result["result"]["contents"]["value"]

        getline = hover("standard.cpp", "getline(input")
        vector = hover("types.cpp", "vector<int>")
        if custom:
            assert "MY GETLINE DESCRIPTION" in getline
            assert "MY VECTOR DESCRIPTION" in vector
            assert "MY MEMBERS" in vector
            assert "Includes descriptions from your user configuration" in getline
            assert "**Example**" not in getline, "An empty override must clear the inherited example"
            assert "**Parameters**" in getline, "Omitted fields must inherit bundled descriptions"
            assert "std::istream& getline" in getline
            assert "[Open full local documentation]" in vector, "Overrides must preserve source links"
        else:
            assert "MY GETLINE DESCRIPTION" not in getline
            assert "Read characters from a stream" in getline
            assert "MY VECTOR DESCRIPTION" not in vector
            assert any("could not read user descriptions" in json.dumps(item) for item in client.notifications)

        push_back = hover("standard.cpp", "push_back(42)")
        assert "Append one element to the end of a vector" in push_back
        assert "Includes compact descriptions maintained by this extension" in push_back
        mapping = hover("types.cpp", "map<int")
        assert "Source: cppreference" in mapping
        assert "Includes compact descriptions" not in mapping
        assert "user configuration" not in mapping
        own = hover("standard.cpp", "printf(0)")
        assert "MY PRINTF DESCRIPTION" not in own and "Source:" not in own
        print("PASS user fields > bundled compact fields > cppreference; project declarations preserved" if custom
              else "PASS invalid descriptions warn and preserve bundled/cppreference hover")
    finally:
        client.close()


def run(binary, clangd, data):
    with tempfile.TemporaryDirectory(prefix="cpp-docs-descriptions-") as temporary:
        descriptions = pathlib.Path(temporary) / "descriptions.json"
        descriptions.write_text(json.dumps([
            {"symbol": "std::getline", "summary": "MY GETLINE DESCRIPTION", "example": ""},
            {"symbol": "std::vector", "summary": "MY VECTOR DESCRIPTION", "members": "MY MEMBERS"},
            {"symbol": "printf", "summary": "MY PRINTF DESCRIPTION"},
        ]))
        check(binary, clangd, data, descriptions, True)
        descriptions.write_text('{"invalid JSON')
        check(binary, clangd, data, descriptions, False)


if __name__ == "__main__":
    run(*sys.argv[1:4])
