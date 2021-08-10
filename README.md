# Write DSP code in Python and run it with Rust
<!--
```bash
python -m venv .env
source .env/bin/activate
pip install maturin
maturin develop
``` -->

```bash
PYO3_PYTHON=python3.9 cargo run
cargo watch -c -s 'PYO3_PYTHON=python3.9 cargo run' -i scripts
```