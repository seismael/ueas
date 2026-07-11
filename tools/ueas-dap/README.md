# UEAS DAP Debug Adapter

VS Code / Zed compatible debug adapter for stepping through UEAS algorithm execution.

## Usage

Add to .vscode/launch.json:

```json
{
    "type": "ueas",
    "request": "launch",
    "program": "library/sorting/quicksort.ueas",
    "debugServer": 4711
}
```
