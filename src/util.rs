use anyhow::{Context, Result};
use fastdivide::DividerU64;
use windows::Win32::{
	Foundation::FILETIME,
	System::{SystemInformation::GetSystemTime, Time::SystemTimeToFileTime},
};

#[static_init::dynamic]
static TIME_DIVIDER: DividerU64 = DividerU64::divide_by(60_u64 * 10_000_000_u64);

/// Converts a timestamp in 100ns intervals into 1 minute intervals.
#[inline]
fn convert_time_to_minutes(time: u64) -> u64 {
	TIME_DIVIDER.divide(time)
}

/// Gets the current time in 100ns intervals since January 1, 1601 (UTC).
unsafe fn current_time() -> Result<FILETIME> {
	let mut file_time = FILETIME::default();
	let system_time = GetSystemTime();
	SystemTimeToFileTime(&system_time, &mut file_time)
		.context("failed to get file time from system time")?;
	Ok(file_time)
}

/// Gets the minutes that have elapsed since the given time.
pub(crate) fn minutes_since(base: FILETIME) -> Result<i64> {
	let base = convert_time_to_minutes(unsafe { std::mem::transmute::<_, u64>(base) }) as i128;
	let now =
		convert_time_to_minutes(unsafe { std::mem::transmute::<_, u64>(current_time()?) }) as i128;
	Ok(now.saturating_sub(base) as i64)
}
