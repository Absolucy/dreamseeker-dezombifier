// SPDX-License-Identifier: 0BSD
#![cfg_attr(
	not(any(debug_assertions, feature = "service")),
	windows_subsystem = "windows"
)]

mod logging;
mod process;
mod terminate;
mod util;
mod window;

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, RecvTimeoutError};
use log::{debug, error, info};
use std::time::Duration;
use windows::Win32::System::{
	ProcessStatus::EnumProcesses,
	Threading::{GetCurrentProcess, SetPriorityClass, BELOW_NORMAL_PRIORITY_CLASS},
};

/// The duration between checks for zombie processes.
const INTERVAL: Duration = Duration::from_secs(90);

fn main_loop(shutdown_rx: Receiver<()>) -> Result<()> {
	unsafe {
		let mut process_ids = vec![0_u32; 1024];
		let mut bytes_returned = 0_u32;
		loop {
			if let Err(err) = EnumProcesses(
				process_ids.as_mut_ptr(),
				(process_ids.len() * std::mem::size_of::<u32>()) as u32,
				&mut bytes_returned,
			) {
				error!("Failed to enumerate processes: {}", err);
				continue;
			}
			let pids = &process_ids[..(bytes_returned as usize / std::mem::size_of::<u32>())];
			for &pid in pids {
				if let Some(process_name) = process::get_process_name(pid)? {
					debug!("pid {pid} = {process_name}");
					if process_name.ends_with("dreamseeker.exe")
						&& !window::has_visible_windows(pid)?
						&& process::mins_since_process_creation(pid)? > 5
					{
						info!("Terminating zombie process {}", pid);
						process::terminate_process(pid)?;
					}
				}
			}
			match shutdown_rx.recv_timeout(INTERVAL) {
				Ok(_) | Err(RecvTimeoutError::Disconnected) => break,
				Err(RecvTimeoutError::Timeout) => (),
			};
		}
		Ok(())
	}
}

#[cfg(windows)]
fn main() -> Result<()> {
	logging::setup_logging().context("failed to setup logging")?;
	info!("Initializing DreamSeeker dezombifier...");
	if let Err(err) = unsafe { SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS) }
	{
		error!("Failed to set priority class to below normal: {}", err);
	}
	info!("Set priority class to below normal");
	let (tx, rx) = crossbeam_channel::bounded::<()>(1);
	terminate::termination_handler(&tx).context("failed to setup termination handler")?;
	main_loop(rx).context("failed to run main loop")?;
	Ok(())
}

#[cfg(not(windows))]
fn main() {
	panic!("This program is only intended to run on Windows.");
}
