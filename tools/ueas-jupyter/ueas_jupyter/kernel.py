"""Stateful multi-cell Jupyter kernel executing UEAS v2.0 cells via the unified `ueas` CLI."""

from __future__ import annotations

import asyncio
import tempfile
from pathlib import Path
from typing import Any

from ipykernel.kernelbase import Kernel


class UeasKernel(Kernel):
    """Stateful Jupyter kernel for UEAS v2.0 supporting `%reset`, `%verify`, and `%transpile`."""

    implementation = "ueas_jupyter"
    implementation_version = "2.0.0"
    language = "ueas"
    language_version = "2.0.0"
    banner = "UEAS v2.0 Stateful Algorithmic Specification & Verification Kernel"

    def __init__(self, **kwargs: Any) -> None:
        super().__init__(**kwargs)  # type: ignore[no-untyped-call]
        self.language_info = {
            "name": "ueas",
            "mimetype": "text/x-ueas",
            "file_extension": ".ueas",
        }
        self._definitions: list[str] = []

    async def do_execute(
        self,
        code: str,
        silent: bool,
        store_history: bool = True,
        user_expressions: dict[str, Any] | None = None,
        allow_stdin: bool = False,
        *,
        cell_meta: dict[str, Any] | None = None,
        cell_id: str | None = None,
    ) -> dict[str, Any]:
        """Execute a UEAS cell with accumulated prior definitions and cell magics."""
        del store_history, user_expressions, allow_stdin, cell_meta, cell_id
        stripped = code.strip()
        if stripped == "%reset":
            self._definitions.clear()
            if not silent:
                self.send_response(  # type: ignore[no-untyped-call]
                    self.iopub_socket, "stream", {"name": "stdout", "text": "Kernel state reset.\n"}
                )
            return {
                "status": "ok",
                "execution_count": self.execution_count,
                "payload": [],
                "user_expressions": {},
            }

        cmd_args = ["run"]
        body = code
        if stripped.startswith("%verify"):
            parts = stripped.splitlines()[0].split()
            level = parts[1] if len(parts) > 1 else "L3"
            cmd_args = ["verify", "--level", level]
            body = "\n".join(stripped.splitlines()[1:])
        elif stripped.startswith("%transpile"):
            parts = stripped.splitlines()[0].split()
            target = parts[1] if len(parts) > 1 else "rust"
            cmd_args = ["transpile", "--target", target]
            body = "\n".join(stripped.splitlines()[1:])

        combined = "\n\n".join([*self._definitions, body]).strip()
        with tempfile.TemporaryDirectory() as tmpdir:
            src_path = Path(tmpdir) / "cell.ueas"
            src_path.write_text(combined, encoding="utf-8")
            proc = await asyncio.create_subprocess_exec(
                "ueas",
                *cmd_args,
                str(src_path),
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout_bytes, stderr_bytes = await asyncio.wait_for(proc.communicate(), timeout=15)
            returncode = proc.returncode if proc.returncode is not None else 1
            if returncode == 0 and any(kw in body for kw in ("Algorithm ", "Record ")):
                self._definitions.append(body)
            output = (stdout_bytes if returncode == 0 else stderr_bytes).decode(
                "utf-8", errors="replace"
            )
            if not silent and output:
                self.send_response(  # type: ignore[no-untyped-call]
                    self.iopub_socket,
                    "stream",
                    {"name": "stdout" if returncode == 0 else "stderr", "text": output},
                )
            return {
                "status": "ok" if returncode == 0 else "error",
                "execution_count": self.execution_count,
                "payload": [],
                "user_expressions": {},
            }


if __name__ == "__main__":
    from ipykernel.kernelapp import IPKernelApp

    IPKernelApp.launch_instance(kernel_class=UeasKernel)
