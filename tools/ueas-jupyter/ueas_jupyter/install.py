"""Install the UEAS v2.0 Jupyter kernel specification."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


def install_kernel_spec() -> Path:
    """Write kernel.json into the user Jupyter kernels directory."""
    spec_dir = Path.home() / ".local" / "share" / "jupyter" / "kernels" / "ueas"
    spec_dir.mkdir(parents=True, exist_ok=True)
    spec: dict[str, Any] = {
        "argv": [sys.executable, "-m", "ueas_jupyter.kernel", "-f", "{connection_file}"],
        "display_name": "UEAS v2.0",
        "language": "ueas",
    }
    target = spec_dir / "kernel.json"
    target.write_text(json.dumps(spec, indent=2), encoding="utf-8")
    return target


if __name__ == "__main__":
    print(install_kernel_spec())
