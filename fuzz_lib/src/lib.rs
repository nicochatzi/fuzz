use pyo3::prelude::*;

#[pyfunction]
pub fn sin(x: f64) -> f64 {
    x.sin()
}

#[pymodule]
fn fuzz(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sin, m)?)?;

    Ok(())
}
