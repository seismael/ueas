from ipykernel.kernelbase import Kernel
import subprocess
import tempfile
import os
import re


class UEASKernel(Kernel):
    implementation = "UEAS"
    implementation_version = "0.1.0"
    language = "python"
    language_version = "3.11"
    banner = "UEAS Kernel v0.1.0 \u2014 Universal Executable Algorithm Standard"

    def do_execute(
        self, code, silent, store_history=True, user_expressions=None, allow_stdin=False
    ):
        code = code.strip()

        if code.startswith("%%ueas"):
            return self._handle_ueas(code)
        else:
            return self._handle_python(code)

    def _handle_ueas(self, code):
        lines = code.split("\n")
        directive = lines[0].replace("%%ueas", "").strip()
        source = "\n".join(lines[1:])

        with tempfile.NamedTemporaryFile(mode="w", suffix=".ueas", delete=False) as f:
            f.write(source)
            tmp = f.name

        try:
            if directive.startswith("transpile"):
                target = "python"
                if "--target" in directive:
                    target = directive.split("--target")[-1].strip()
                result = subprocess.run(
                    ["ueas", "transpile", tmp, "--target", target],
                    capture_output=True,
                    text=True,
                    timeout=10,
                )
                if result.returncode == 0:
                    self.send_response(
                        self.iopub_socket,
                        "stream",
                        {"name": "stdout", "text": result.stdout},
                    )
                else:
                    self.send_response(
                        self.iopub_socket,
                        "stream",
                        {"name": "stderr", "text": result.stderr},
                    )
            elif directive == "check" or not directive:
                result = subprocess.run(
                    ["ueas", "check", tmp], capture_output=True, text=True, timeout=10
                )
                if result.returncode == 0:
                    self.send_response(
                        self.iopub_socket,
                        "stream",
                        {"name": "stdout", "text": result.stdout},
                    )
                else:
                    self.send_response(
                        self.iopub_socket,
                        "stream",
                        {"name": "stderr", "text": result.stderr},
                    )
            elif directive == "run":
                result = subprocess.run(
                    ["ueas", "run", tmp], capture_output=True, text=True, timeout=10
                )
                if result.returncode == 0:
                    output = result.stdout
                    # Emit rich HTML complexity profile
                    self._emit_run_result(output, source)
                else:
                    self.send_response(
                        self.iopub_socket,
                        "stream",
                        {"name": "stderr", "text": result.stderr},
                    )
        finally:
            os.unlink(tmp)

        return {
            "status": "ok",
            "execution_count": self.execution_count,
            "payload": [],
            "user_expressions": {},
        }

    def _handle_python(self, code):
        try:
            exec(code, self.user_ns)
        except Exception as e:
            self.send_response(
                self.iopub_socket, "stream", {"name": "stderr", "text": str(e)}
            )
        return {
            "status": "ok",
            "execution_count": self.execution_count,
            "payload": [],
            "user_expressions": {},
        }

    def _emit_run_result(self, raw_output, source):
        """Emit a rich HTML complexity profile card alongside text output."""
        metrics = self._parse_run_output(raw_output)
        algo_name = metrics.get("algorithm", "unknown")
        step_count = metrics.get("step_count", "?")
        heap_bytes = metrics.get("heap", "?")
        exit_code = metrics.get("exit_code", "?")

        # Extract complexity from the source
        complexity = "?"
        for line in source.split("\n"):
            m = re.search(r'Complexity:\s*"([^"]+)"', line)
            if m:
                complexity = m.group(1)
                break

        html = f"""<style>
.ueas-profile {{ font-family: 'Segoe UI', system-ui, sans-serif; border: 1px solid #30363d; border-radius: 8px; margin: 12px 0; overflow: hidden; }}
.ueas-profile .header {{ background: #161b22; padding: 10px 16px; border-bottom: 1px solid #30363d; display: flex; justify-content: space-between; align-items: center; }}
.ueas-profile .header .title {{ font-size: 1.1em; font-weight: 600; color: #c9d1d9; }}
.ueas-profile .header .badge {{ font-size: 0.7em; padding: 2px 8px; border-radius: 10px; font-family: monospace; }}
.ueas-profile .badge.ok {{ background: #1f6feb; color: white; }}
.ueas-profile .badge.fail {{ background: #da3633; color: white; }}
.ueas-profile .body {{ padding: 14px 16px; background: #0d1117; }}
.ueas-profile .metric {{ display: inline-block; margin: 4px 20px 4px 0; }}
.ueas-profile .metric .label {{ font-size: 0.7em; color: #8b949e; text-transform: uppercase; letter-spacing: 0.05em; }}
.ueas-profile .metric .value {{ font-size: 1.2em; font-weight: 600; color: #c9d1d9; font-family: monospace; }}
.ueas-profile .complexity {{ color: #7ee787; font-family: monospace; font-size: 0.85em; }}
.ueas-profile .source {{ margin-top: 12px; padding: 12px; background: #161b22; border-radius: 6px; font-family: 'Cascadia Code', monospace; font-size: 0.85em; color: #d2a8ff; white-space: pre-wrap; overflow-x: auto; }}
</style>
<div class="ueas-profile">
  <div class="header">
    <span class="title">{algo_name}</span>
    <span class="badge ok">exit={exit_code}</span>
  </div>
  <div class="body">
    <div class="metric"><div class="label">Step Count</div><div class="value">{step_count}</div></div>
    <div class="metric"><div class="label">Heap Bytes</div><div class="value">{heap_bytes}</div></div>
    <div class="metric"><div class="label">Complexity</div><div class="value complexity">{complexity}</div></div>
    <div class="source">{source.strip()}</div>
  </div>
</div>"""

        self.send_response(
            self.iopub_socket,
            "display_data",
            {
                "data": {"text/html": html},
                "metadata": {},
            },
        )
        self.send_response(
            self.iopub_socket,
            "stream",
            {
                "name": "stdout",
                "text": raw_output,
            },
        )

    def _parse_run_output(self, raw):
        """Parse 'ueas run' output into a metrics dict."""
        metrics = {}
        for line in raw.split("\n"):
            m = re.match(r"\s+algorithm:\s+(.+)", line)
            if m:
                metrics["algorithm"] = m.group(1)
            m = re.match(r"\s+step_count:\s+(\d+)", line)
            if m:
                metrics["step_count"] = m.group(1)
            m = re.match(r"\s+heap_allocated:\s+(\d+)\s+bytes", line)
            if m:
                metrics["heap"] = m.group(1)
            m = re.match(r"OK\s+.*exit_code=(\w+)", line)
            if m:
                metrics["exit_code"] = m.group(1)
        return metrics


if __name__ == "__main__":
    from ipykernel.kernelapp import IPKernelApp

    IPKernelApp.launch_instance(kernel_class=UEASKernel)
