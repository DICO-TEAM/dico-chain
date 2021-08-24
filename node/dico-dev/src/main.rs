mod chain_spec;
#[macro_use]
mod service;
pub mod executor;
pub mod rpc;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod command;

#[cfg(feature = "cli")]
pub use cli::*;
#[cfg(feature = "cli")]
pub use command::*;

fn main() -> sc_cli::Result<()> {
	command::run()
}
