use anyhow::Result;
use log::debug;
use windows::Win32::{
	Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE},
	UI::WindowsAndMessaging::{
		EnumWindows, GetWindow, GetWindowThreadProcessId, IsWindowVisible, GW_OWNER,
	},
};

pub(crate) fn has_visible_windows(pid: u32) -> Result<bool> {
	#[derive(Debug)]
	struct Info {
		pid: u32,
		has_visible_windows: bool,
	}

	extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
		unsafe {
			let mut process_id = 0u32;
			if GetWindowThreadProcessId(hwnd, Some(&mut process_id)) == 0 {
				return TRUE;
			}
			let info = &mut *std::mem::transmute::<LPARAM, *mut Info>(lparam);
			if (process_id == info.pid as u32)
				&& IsWindowVisible(hwnd) == TRUE
				&& GetWindow(hwnd, GW_OWNER).is_ok()
			{
				debug!("Found visible window for pid {}", info.pid);
				info.has_visible_windows = true;
				return FALSE;
			}
			TRUE
		}
	}

	let mut info = Info {
		pid,
		has_visible_windows: false,
	};

	unsafe {
		let _ = EnumWindows(
			Some(enum_windows_proc),
			std::mem::transmute::<*mut Info, LPARAM>(&mut info as *mut Info),
		);
	}

	Ok(info.has_visible_windows)
}
