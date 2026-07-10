from ipykernel.kernelbase import Kernel
import subprocess
import tempfile
import os


class UEASKernel(Kernel):
    implementation = "UEAS"
    implementation_version = "0.1.0"
    language = "python"
    language_version = "3.11"
    banner = "UEAS Kernel — Universal Executable Algorithm Standard"

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


if __name__ == "__main__":
    from ipykernel.kernelapp import IPKernelApp

    IPKernelApp.launch_instance(kernel_class=UEASKernel)
