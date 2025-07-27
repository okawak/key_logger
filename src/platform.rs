#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::start_key_monitoring;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::start_key_monitoring;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::start_key_monitoring;

// Fallback for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub(crate) fn start_key_monitoring(
    _stats: crate::stats::KeyStatistics,
) -> crate::error::Result<()> {
    Err(crate::error::KeyLoggerError::PlatformNotSupported)
}
