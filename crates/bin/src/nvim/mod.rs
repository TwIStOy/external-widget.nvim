mod handler;

pub use handler::NeovimHandler;

/// Start a TCP server on the given address.
pub(crate) fn start_server(addr: &str) -> anyhow::Result<()> {}
