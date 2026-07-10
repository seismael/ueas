import json
import os
import sys


def main():
    kernel_dir = os.path.join(os.path.dirname(__file__), "..")
    kernel_spec = {
        "argv": [
            sys.executable,
            "-m",
            "ueas_jupyter.kernel",
            "-f",
            "{connection_file}",
        ],
        "display_name": "UEAS (Algorithm Standard)",
        "language": "python",
    }

    import jupyter_client

    kd = jupyter_client.kernelspec.KernelSpecManager().install_kernel_spec(
        kernel_dir, "ueas", user=True
    )
    print(f"UEAS kernel installed to {kd}")


if __name__ == "__main__":
    main()
