use crate::{error::Result, platform_common};

pub(crate) fn start_key_monitoring(stats: crate::stats::KeyStatistics) -> Result<()> {
    platform_common::start_key_monitoring(stats)
}
