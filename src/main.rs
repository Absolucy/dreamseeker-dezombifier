// SPDX-License-Identifier: 0BSD
#![cfg_attr(
	not(any(debug_assertions, feature = "service")),
	windows_subsystem = "windows"
)]

mod logging;
#[cfg(all(not(debug_assertions), feature = "service"))]
mod service;

use anyhow::{Context, Result};
use log::info;
use windows::Win32::System::Threading::{
	GetCurrentProcess, SetPriorityClass, BELOW_NORMAL_PRIORITY_CLASS,
};

#[cfg(windows)]
fn main() -> Result<()> {
	logging::setup_logging().context("failed to setup logging")?;
	info!("Initializing DreamSeeker dezombifier...");
	unsafe { SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS) }
		.context("failed to set priority class")?;
	info!("Set priority class to below normal");
	cfg_if::cfg_if! {
		if #[cfg(all(feature = "service", not(debug_assertions)))] {
			info!("Running as a service");
			service::run().context("failed to run service")?;
		} else {
			info!("Running normally");
			let (tx, rx) = crossbeam_channel::bounded::<()>(1);
			dreamseeker_dezombifier::terminate::termination_handler(&tx)
				.context("failed to setup termination handler")?;
			dreamseeker_dezombifier::main_loop(rx).context("failed to run main loop")?;
		}
	}
	Ok(())
}

#[cfg(not(windows))]
fn main() {
	panic!("This program is only intended to run on Windows.");
}
