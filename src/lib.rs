use pyo3::prelude::*;

mod command;

#[pymodule]
fn _oxi_nixinfra(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<command::CommandResult>()?;
    Ok(())
}
