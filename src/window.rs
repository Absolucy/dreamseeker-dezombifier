use crate::{util::minutes_since, VISIBLE_PIDS};
use log::{debug, error, info};
use scopeguard::defer;
use windows::Win32::{
	Foundation::{CloseHandle, BOOL, FALSE, FILETIME, HWND, LPARAM, MAX_PATH, TRUE},
	System::{
		ProcessStatus::GetModuleFileNameExW,
		Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
	},
	UI::WindowsAndMessaging::{GetWindowThreadProcessId, IsWindowVisible},
};

pub(crate) unsafe extern "system" fn enum_window(window: HWND, _: LPARAM) -> BOOL {
	let mut pid = 0;
	debug!("getting pid for window {window:?}");
	if GetWindowThreadProcessId(window, Some(&mut pid)) == 0 {
		return FALSE;
	}
	debug!("got pid {pid} for window {window:?}");
	if VISIBLE_PIDS.with_borrow(|pids| pids.contains(&pid)) {
		return TRUE;
	}
	let handle = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid) {
		Ok(handle) => handle,
		Err(err) => {
			error!("Failed to open process {pid}: {err:?}");
			return TRUE;
		}
	};
	debug!("opened process {pid}");
	defer! {
		debug!("[{pid}] closing process (handle {handle:?})");
		if let Err(err) = CloseHandle(handle) {
			error!("Failed to close handle for process {pid}: {err:?}");
		}
	}
	let mut creation_time = FILETIME::default();
	let mut _dummy_a = FILETIME::default();
	let mut _dummy_b = FILETIME::default();
	let mut _dummy_c = FILETIME::default();
	debug!("getting creation time for process {pid}");
	if GetProcessTimes(
		handle,
		&mut creation_time,
		&mut _dummy_a,
		&mut _dummy_b,
		&mut _dummy_c,
	)
	.is_err()
	{
		return TRUE;
	}
	debug!("got creation time for process {pid}");
	// Don't kill processes that've been alive for less than 5 minutes
	if minutes_since(creation_time).unwrap() < 5 {
		VISIBLE_PIDS.with_borrow_mut(|pids| pids.insert(pid));
		return TRUE;
	}
	debug!("[{pid}] getting filename");
	let mut filename = [0_u16; MAX_PATH as usize];
	let length = GetModuleFileNameExW(handle, None, &mut filename) as usize;
	if length == 0 {
		error!("Failed to get module filename for process {pid}");
		return TRUE;
	}
	let filename = stfu8::encode_u16_pretty(&filename[..length]);
	debug!("[{pid}] filename is {filename}");
	if filename.contains("dreamseeker.exe") {
		debug!("Found DreamSeeker.exe window: {window:?}, pid={pid}, filename={filename}");
		if IsWindowVisible(window) == TRUE {
			info!("Found visible DS window: hwnd={window:?}, pid={pid}, filename={filename}");
			VISIBLE_PIDS.with_borrow_mut(|pids| pids.insert(pid));
		} else {
			info!("Found zombie DS window: hwnd={window:?}, pid={pid}, filename={filename}");
		}
	}
	TRUE
}
