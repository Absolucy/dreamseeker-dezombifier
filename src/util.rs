use anyhow::{Context, Result};
use fastdivide::DividerU64;
use std::sync::LazyLock;
use windows::Win32::{
	Foundation::FILETIME,
	System::{SystemInformation::GetSystemTime, Time::SystemTimeToFileTime},
};

static TIME_DIVIDER: LazyLock<DividerU64> =
	LazyLock::new(|| DividerU64::divide_by(60_u64 * 10_000_000_u64));

/// Converts a timestamp in 100ns intervals into 1 minute intervals.
#[inline]
fn convert_time_to_minutes(time: u64) -> u64 {
	TIME_DIVIDER.divide(time)
}

/// Gets the current time in 100ns intervals since January 1, 1601 (UTC).
unsafe fn current_time() -> Result<u64> {
	let mut file_time = FILETIME::default();
	let system_time = GetSystemTime();
	SystemTimeToFileTime(&system_time, &mut file_time)
		.context("failed to get file time from system time")?;
	Ok(((file_time.dwHighDateTime as u64) << 32) | (file_time.dwLowDateTime as u64))
}

/// Gets the minutes that have elapsed since the given time.
pub(crate) fn minutes_since(base: FILETIME) -> Result<i64> {
	let base =
		convert_time_to_minutes(((base.dwHighDateTime as u64) << 32) | (base.dwLowDateTime as u64))
			as i128;
	let now = unsafe { current_time() }.map(convert_time_to_minutes)? as i128;
	Ok(now.saturating_sub(base) as i64)
}
