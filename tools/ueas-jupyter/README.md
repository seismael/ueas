# UEAS Jupyter Kernel

Interactive Jupyter notebook kernel for UEAS. Supports mixed Python+UEAS cells.

## Install

```
pip install -e .
python -m ueas_jupyter.install
```

## Usage

```
%%ueas run
Algorithm Test(x)
    Require: x: Integer
    Ensure: Integer
    Complexity: "O(1)"
    return x

%%ueas transpile --target python
Algorithm Test(x)
    Require: x: Integer
    Ensure: Integer
    Complexity: "O(1)"
    return x
```
