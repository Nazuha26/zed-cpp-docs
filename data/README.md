# Compact documentation

`compact-docs.json` contains 100 original, concise descriptions: 60 C++ symbols
and 40 C library symbols. The selection focuses on everyday containers, strings,
algorithms, formatted and file I/O, memory, byte strings, conversions, types,
public pair fields and debugging macros.

Each description has a complete runnable example. C++ examples target C++17.
The 40 C examples also compile and run as C11. Signatures show representative
overloads; version differences and important preconditions are stated in the
text. Template parameter names in signatures are placeholders, not additional
types that a user must declare.

## Sources

[`compact-docs-sources.json`](compact-docs-sources.json) records, for every symbol:

- `cppreference`: the matching page in the downloaded cppreference HTML book
  dated 2025-02-09. These URLs are checked against the imported index.
- `standard`: a C++ working-draft section or a section of the C11 committee draft
  N1570, used to check behavior, return values and preconditions.
- `language`: `cpp` for C++ library symbols, or `c` for the C library subset.

The [C++ working draft](https://eel.is/c++draft/) includes newer facilities and
overloads. The compact examples deliberately stay within C++17; the tooltip
calls out relevant version differences, such as the return type of
`vector::emplace_back` and mutable `basic_string::data`.
The [C11 draft text](https://port70.net/~nsz/c/c11/n1570.html) supplies the C
baseline. C++ const-preserving overloads of C string search functions are
identified separately in their descriptions.

The source manifest is for maintenance and verification. Runtime descriptions
retain the imported page, headers and attribution, so the full documentation
still opens locally and works offline. Content licenses are listed in
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md).

## Verification

```sh
python tests/examples.py clang++ --c-compiler clang --run
python tests/catalog.py target/release/zed-cpp-docs-proxy clangd /path/to/local/docs
```

The first command compiles and runs all 100 examples as C++17 and the 40 C
examples again as C11. The second checks all 100 descriptions against the imported
index and requests each tooltip through a real clangd process. It verifies the
curated text, attribution and local documentation link. User overrides and
unmodified cppreference fallbacks are tested separately in `tests/descriptions.py`.
