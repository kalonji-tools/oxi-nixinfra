use pyo3::prelude::*;

mod backend;
mod command;
mod helpers;
mod host;

#[pymodule]
fn _oxi_nixinfra(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<command::CommandResult>()?;
    m.add_class::<host::Host>()?;
    m.add_class::<host::AsyncHost>()?;
    Ok(())
}
