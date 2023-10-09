use crate::VISIBLE_PIDS;
use anyhow::{Context, Result};
use log::{debug, error, info};
use scopeguard::defer;
use windows::Win32::{
	Foundation::{CloseHandle, FALSE, MAX_PATH},
	System::{
		ProcessStatus::{EnumProcesses, GetModuleFileNameExW},
		Threading::{
			OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE,
			PROCESS_VM_READ,
		},
	},
};

pub(crate) unsafe fn enum_process(processes: &mut Vec<u32>) -> Result<()> {
	const LENGTH: u32 = (std::mem::size_of::<u32>() * 1024) as u32;

	let mut cb_needed: u32 = 0;
	EnumProcesses(processes.as_mut_ptr(), LENGTH, &mut cb_needed)
		.context("failed to enumerate processes")?;
	processes.set_len((cb_needed / std::mem::size_of::<u32>() as u32) as usize);
	for pid in processes.iter() {
		debug!("opening process {pid}");
		let handle = match OpenProcess(
			PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE,
			FALSE,
			*pid,
		) {
			Ok(handle) => handle,
			Err(err) => {
				error!("Failed to open process {pid}: {err:?}");
				continue;
			}
		};
		debug!("opened process {pid}");
		defer! {
			debug!("closing process {pid} (handle {handle:?})");
			if let Err(err) = CloseHandle(handle) {
				error!("Failed to close handle for process {pid}: {err:?}");
			}
		}
		let mut filename = [0_u16; MAX_PATH as usize];
		debug!("{pid}: getting filename");
		let length = GetModuleFileNameExW(handle, None, &mut filename) as usize;
		if length == 0 {
			error!("Failed to get module filename for process {pid}");
			continue;
		}
		let filename = stfu8::encode_u16_pretty(&filename[..length]);
		debug!("{pid}: filename={filename}");
		if !filename.contains("dreamseeker.exe")
			|| VISIBLE_PIDS.with_borrow(|pids| pids.contains(pid))
		{
			continue;
		}
		debug!("{pid}: filename={filename}");
		match TerminateProcess(handle, 0) {
			Ok(_) => info!("killed zombie process {filename} (pid={pid})"),
			Err(err) => error!("failed to kill zombie process {filename} (pid={pid}): {err:?}"),
		}
	}
	Ok(())
}
