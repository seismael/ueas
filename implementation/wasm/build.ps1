$ErrorActionPreference = "Stop"
Write-Host "Building UEAS WASM kernel..."
wasm-pack build --target web
Write-Host "Copying WASM artifacts to Cloudflare Workers MCP server..."
$cfSrc = (Resolve-Path "$PSScriptRoot\..\tools\ueas-mcp-cf\src").Path
Copy-Item -Force "$PSScriptRoot\pkg\ueas_wasm.js" $cfSrc
Copy-Item -Force "$PSScriptRoot\pkg\ueas_wasm_bg.wasm" $cfSrc
Write-Host "Done. WASM deployed to CF Worker."
