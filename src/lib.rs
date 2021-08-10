use pyo3::prelude::*;

#[pyfunction]
pub fn hello() {
    println!("Hello world!");
}

#[pymodule]
fn fuzz(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello, m)?)?;

    Ok(())
}
