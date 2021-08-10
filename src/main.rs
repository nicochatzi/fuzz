use pyo3::prelude::*;
use pyo3::{py_run, PyCell, PyObjectProtocol};

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let path = format!("{}/scripts/dsp.py", std::env!("CARGO_MANIFEST_DIR"));
        let code = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
        let dsp = PyModule::from_code(py, &code, "dsp.py", "scripts")?;

        dsp.getattr("init")?.call0()?;
        dsp.getattr("update")?.call1((1.0,))?;
        dsp.getattr("process")?.call1((2.0,))?;

        Ok(())
    })
}
