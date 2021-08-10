use pyo3::prelude::*;
use pyo3::{py_run, PyCell, PyObjectProtocol};

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let path = format!("{}/scripts/tri.py", std::env!("CARGO_MANIFEST_DIR"));
        let code = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
        let dsp = PyModule::from_code(py, &code, "tri.py", "scripts")?;

        let proc = dsp.getattr("Processor")?.call0()?;
        proc.call_method1("update", (256, 44_000))?;
        let buffer: Vec<f64> = proc.call_method0("process")?.extract()?;
        println!("buffer: {:?}", buffer);

        Ok(())
    })
}
