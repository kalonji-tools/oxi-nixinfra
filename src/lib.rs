use pyo3::prelude::*;

mod backend;
mod command;
mod helpers;
mod host;
mod modules;
mod parse;

#[pymodule]
fn _oxi_nixinfra(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<command::CommandResult>()?;
    m.add_class::<host::Host>()?;
    m.add_class::<host::AsyncHost>()?;
    m.add_class::<modules::service::Service>()?;
    m.add_class::<modules::service::AsyncService>()?;
    m.add_class::<modules::file::File>()?;
    m.add_class::<modules::file::AsyncFile>()?;
    m.add_class::<modules::user::User>()?;
    m.add_class::<modules::user::AsyncUser>()?;
    m.add_class::<modules::system_info::SystemInfo>()?;
    m.add_class::<modules::system_info::AsyncSystemInfo>()?;
    m.add_class::<modules::nix_package::NixPackage>()?;
    m.add_class::<modules::nix_package::AsyncNixPackage>()?;
    m.add_class::<modules::process::Process>()?;
    m.add_class::<modules::process::AsyncProcess>()?;
    m.add_class::<modules::socket::Socket>()?;
    m.add_class::<modules::socket::AsyncSocket>()?;
    m.add_class::<modules::mountpoint::MountPoint>()?;
    m.add_class::<modules::mountpoint::AsyncMountPoint>()?;
    m.add_class::<modules::sysctl::Sysctl>()?;
    m.add_class::<modules::sysctl::AsyncSysctl>()?;
    m.add_class::<modules::environment::Environment>()?;
    m.add_class::<modules::environment::AsyncEnvironment>()?;
    Ok(())
}
