$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $projectRoot 'crates/proxy/Cargo.toml'
$target = Join-Path $projectRoot 'target'
$installRoot = if ($env:CARGO_INSTALL_ROOT) { $env:CARGO_INSTALL_ROOT }
    elseif ($env:CARGO_HOME) { $env:CARGO_HOME }
    else { Join-Path $HOME '.cargo' }
$installRoot = [System.IO.Path]::GetFullPath($installRoot)
$bin = Join-Path $installRoot 'bin'
$installed = Join-Path $bin 'zed-cpp-docs-proxy.exe'
$backup = Join-Path $bin ('zed-cpp-docs-proxy.previous-' + [guid]::NewGuid().ToString('N') + '.exe')

# Build before touching the executable used by an open editor.
& cargo build --manifest-path $manifest --release --locked --target-dir $target
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Windows permits renaming a running executable, but cannot overwrite it.
# The old server keeps running until the user restarts the language server.
if (Test-Path -LiteralPath $installed) {
    Move-Item -LiteralPath $installed -Destination $backup
}
try {
    & cargo install --path (Join-Path $projectRoot 'crates/proxy') --locked --force --offline --target-dir $target --root $installRoot
    if ($LASTEXITCODE -ne 0) { throw 'Cargo could not install the new proxy.' }
} catch {
    if ((Test-Path -LiteralPath $backup) -and !(Test-Path -LiteralPath $installed)) {
        Move-Item -LiteralPath $backup -Destination $installed
    }
    throw
}

# A backup still used by a running server is removed on a later successful build.
Get-ChildItem -LiteralPath $bin -Filter 'zed-cpp-docs-proxy.previous-*.exe' -File | ForEach-Object {
    try { Remove-Item -LiteralPath $_.FullName -ErrorAction Stop } catch { }
}
Write-Host 'Proxy updated. In Zed, run: editor: restart language server'
