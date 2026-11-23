# UEAS v2.0 DAP Debug Adapter

VS Code / Zed compatible Debug Adapter Protocol (`Content-Length` framed stdio) server backed by `ueas-vm` with breakpoints, locals inspection, and `stepBack` time-travel debugging.

## Usage

Invoke via the unified CLI (`ueas dap`) or directly via `ueas-dap`:

```json
{
  "type": "ueas",
  "request": "launch",
  "name": "Debug UEAS Algorithm",
  "program": "${workspaceFolder}/stdlib/sorting/quicksort.ueas"
}
```
