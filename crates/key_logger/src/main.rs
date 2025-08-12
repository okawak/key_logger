mod config;
mod error;
mod export;
mod platform;
mod platform_common;
mod stats;

use anyhow::Result;
use log::{debug, error, info};
use std::{io::Write, sync::Arc};

fn main() -> Result<()> {
    // Initialize logger - defaults to RUST_LOG if set, otherwise INFO
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    let config = config::Config::from_env()?;
    info!("Key Logger starting...");
    info!("Press Ctrl+C to stop and save statistics");

    match &config.output_dir {
        Some(dir) => info!("Output directory: {}", dir.display()),
        None => info!("Output directory: (current working directory)"), // This case should no longer occur with default csv dir
    }

    let statistics = stats::create_statistics();
    platform_common::setup_exit_handler()?;

    let result = platform::start_key_monitoring(Arc::clone(&statistics));
    match result {
        Ok(()) if platform_common::should_exit() => {
            info!("Received exit signal, saving statistics...");
        }
        Ok(()) => {
            info!("Monitoring completed, saving statistics...");
        }
        Err(e) => {
            error!("Error during monitoring: {e}");
        }
    }

    save_and_exit(&statistics, &config);
}

fn save_and_exit(statistics: &stats::KeyStatistics, config: &config::Config) -> ! {
    info!("Saving statistics...");

    let result = save_statistics_internal(statistics, config);

    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            error!("Error: {e}");

            // Provide helpful hints for common errors
            use std::io::ErrorKind;
            for cause in e.chain().skip(1) {
                if let Some(ioe) = cause.downcast_ref::<std::io::Error>() {
                    match ioe.kind() {
                        ErrorKind::PermissionDenied => {
                            error!(
                                "Hint: Run in a writable directory or set KEY_LOGGER_OUTPUT_DIR."
                            );
                            break;
                        }
                        ErrorKind::OutOfMemory | ErrorKind::WriteZero => {
                            error!("Hint: Check if the output directory is writable.");
                            break;
                        }
                        ErrorKind::Other if format!("{ioe}").contains("No space left") => {
                            error!("Hint: Check available disk space.");
                            break;
                        }
                        _ => {}
                    }
                }
            }
            let _ = std::io::stderr().flush();
            std::process::exit(1);
        }
    }
}

fn save_statistics_internal(
    statistics: &stats::KeyStatistics,
    config: &config::Config,
) -> Result<()> {
    let stats_snapshot = stats::get_statistics_snapshot(statistics)?;
    if stats_snapshot.is_empty() {
        info!("No key presses recorded.");
        return Ok(());
    }

    if let Some(ref dir) = config.output_dir {
        debug!("Exporting to directory: {}", dir.display());
    }
    let path = export::export_to_csv_with_path(&stats_snapshot, config.output_dir.as_deref())?;
    info!("Statistics saved to: {}", path.display());

    let total_keys: u64 = stats_snapshot.values().copied().sum();
    let unique_keys = stats_snapshot.len();
    info!("Total key presses: {total_keys}");
    info!("Unique keys pressed: {unique_keys}");

    // Top 10 most pressed keys
    let mut sorted_keys: Vec<_> = stats_snapshot.iter().collect();
    sorted_keys.sort_unstable_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    info!("Top 10 most pressed keys:");
    for (i, (key, count)) in sorted_keys.iter().take(10).enumerate() {
        info!("{}. {}: {}", i + 1, key, count);
    }

    debug!("Total entries exported: {}", stats_snapshot.len());

    Ok(())
}
