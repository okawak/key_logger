use crate::{error::Result, platform_common};
use log::{info, warn};

// External C functions for accessibility check
#[allow(non_snake_case)]
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> core::ffi::c_char;
}

#[inline]
fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

pub(crate) fn start_key_monitoring(stats: crate::stats::KeyStatistics) -> Result<()> {
    // Check accessibility permissions (informational only)
    let is_trusted = is_accessibility_trusted();
    if !is_trusted {
        warn!("Accessibility permissions not granted.");
        warn!("Some keyboard monitoring features may be limited.");
        info!("For full functionality, enable accessibility permissions:");
        info!("System Settings > Privacy & Security > Security > Accessibility");
        info!("Add Terminal application and restart.");
    }

    // Use the common implementation
    platform_common::start_key_monitoring(stats)
}
