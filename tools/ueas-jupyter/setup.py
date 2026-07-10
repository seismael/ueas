from setuptools import setup, find_packages

setup(
    name="ueas-jupyter",
    version="0.1.0",
    packages=find_packages(),
    install_requires=["ipykernel", "jupyter"],
    entry_points={
        "console_scripts": [
            "ueas-jupyter-install = ueas_jupyter.install:main",
        ],
    },
)
