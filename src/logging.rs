use anyhow::{Context, Result};

/// Sets up logging for the application.
pub(crate) fn setup_logging() -> Result<()> {
	cfg_if::cfg_if! {
		if #[cfg(debug_assertions)] {
			simple_logger::init_with_level(log::Level::Debug)
				.context("failed to initialize simple logger")
		} else if #[cfg(feature = "service")] {
			eventlog::init("DreamSeeker Dezombifier", log::Level::Info)
				.context("failed to initialize Windows event logger")
		} else {
			eprintln!("Warning: no logger!");
			Ok(())
		}
	}
}
