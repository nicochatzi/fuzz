# Write DSP code in Python and run it with Rust


Run the Rust audio app with:

```bash
cd fuzz_app
PYO3_PYTHON=python3.9 cargo run
```

Now you can write your dsp code in `scripts/dsp.py` and it will be hotreloaded by the fuzz_app.
The app expects a `Processor` class and will call `__init__()`, `update()` and `process()`.

Example processor that returns a buffer full of `0`

```py

class Processor:
    def __init__(self):
        self.buffer_size = 0

    def update(self, buffer_size, sample_rate):
        self.buffer_size = buffer_size

    def process(self):
        return [0 for i in range(self.buffer_size)]

```

## Building a Python module written in Rust

Build the rust-based python module with:


```bash
cd fuzz_lib
python -m venv .env
source .env/bin/activate
pip install maturin
maturin develop
# test calling a Rust function from Python with:
python ../scripts/with_lib.py
```
