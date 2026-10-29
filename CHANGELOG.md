# Changelog

## [0.1.0] - 2026-10-29

First release of C/C++ Docs for Zed.

- Replace standard C/C++ symbol hover tooltips with readable cppreference documentation, signatures and examples.
- Download cppreference on first start and keep documentation and full browser pages available offline.
- Preserve ordinary clangd completion, diagnostics, navigation and tooltips for project symbols.
- Support custom descriptions through `descriptions.json`.
- Download prebuilt proxies automatically on Windows x86_64 and Linux x86_64, including WSL.

### Installation

Install `clangd`, `curl` and Rust, clone this repository at `v0.1.0`, then use
**Extensions → Install Dev Extension** in Zed and select the cloned directory.
Apply the language server settings from the [README](https://github.com/Nazuha26/zed-cpp-docs/blob/v0.1.0/README.md#installation).
The extension downloads the matching proxy from this release automatically.

The proxy archives contain the executable at their root. `cpp-docs-extension.wasm`
is the compiled Zed launcher; installing the dev extension from source remains the
documented installation method. `SHA256SUMS` contains checksums for all three assets.
