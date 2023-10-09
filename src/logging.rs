use anyhow::{Context, Result};

/// Sets up logging for the application.
pub(crate) fn setup_logging() -> Result<()> {
	simple_logger::init_with_env().context("failed to initialize simple logger")
}
