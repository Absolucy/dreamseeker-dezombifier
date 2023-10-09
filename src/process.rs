use crate::util::minutes_since;
use anyhow::{Context, Result};
use log::error;
use scopeguard::defer;
use windows::Win32::{
	Foundation::{CloseHandle, FALSE, FILETIME, MAX_PATH},
	System::{
		ProcessStatus::GetModuleFileNameExW,
		Threading::{
			GetProcessTimes, OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION,
			PROCESS_TERMINATE, PROCESS_VM_READ,
		},
	},
};

pub(crate) fn mins_since_process_creation(pid: u32) -> Result<i64> {
	unsafe {
		let handle = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid) {
			Ok(handle) => handle,
			Err(_) => return Ok(0),
		};
		defer! {
			CloseHandle(handle).expect("failed to close pid");
		}

		let mut creation_time = FILETIME::default();
		let mut _dummy_a = FILETIME::default();
		let mut _dummy_b = FILETIME::default();
		let mut _dummy_c = FILETIME::default();
		GetProcessTimes(
			handle,
			&mut creation_time,
			&mut _dummy_a,
			&mut _dummy_b,
			&mut _dummy_c,
		)
		.with_context(|| format!("failed to get process times for pid {pid}"))?;

		minutes_since(creation_time)
			.with_context(|| format!("failed to get minutes since process creation for pid {pid}"))
	}
}

pub(crate) fn get_process_name(pid: u32) -> Result<Option<String>> {
	unsafe {
		let handle = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid) {
			Ok(handle) => handle,
			Err(_) => return Ok(None),
		};

		defer! {
			CloseHandle(handle).expect("failed to close handle");
		}

		let mut exe_name = [0_u16; MAX_PATH as usize];
		let length = GetModuleFileNameExW(handle, None, &mut exe_name) as usize;
		if length == 0 {
			error!("Failed to get module filename for process {pid}");
			Ok(None)
		} else {
			Ok(Some(stfu8::encode_u16_pretty(&exe_name[..length])))
		}
	}
}

pub(crate) fn terminate_process(pid: u32) -> Result<()> {
	unsafe {
		let handle = OpenProcess(PROCESS_TERMINATE, FALSE, pid)
			.with_context(|| format!("failed to open pid {pid} for termination"))?;
		defer! {
			CloseHandle(handle).expect("failed to close");
		}
		TerminateProcess(handle, 0).with_context(|| format!("failed to terminate pid {pid}"))?;
		Ok(())
	}
}
