pub mod process;
pub mod terminate;
pub mod util;
pub mod window;

use ahash::AHashSet;
use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, RecvTimeoutError};
use std::{cell::RefCell, time::Duration};
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;

thread_local! {
	pub(crate) static VISIBLE_PIDS: RefCell<AHashSet<u32>> = RefCell::default();
}

/// The duration between checks for zombie processes.
const INTERVAL: Duration = Duration::from_secs(90);

#[doc(hidden)]
pub fn main_loop(shutdown_rx: Receiver<()>) -> Result<()> {
	unsafe {
		let mut processes: Vec<u32> = Vec::with_capacity(1024);
		loop {
			VISIBLE_PIDS.with_borrow_mut(|pids| pids.clear());
			EnumWindows(Some(window::enum_window), None).context("Failed to enumerate windows")?;
			process::enum_process(&mut processes)
				.context("failed to scan processes for zombies")?;
			match shutdown_rx.recv_timeout(INTERVAL) {
				Ok(_) | Err(RecvTimeoutError::Disconnected) => break,
				Err(RecvTimeoutError::Timeout) => (),
			};
		}
		Ok(())
	}
}
