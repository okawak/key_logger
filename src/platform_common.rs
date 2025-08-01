use crate::{error::Result, stats};
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
use device_query::{DeviceQuery, DeviceState, Keycode};
use log::{debug, error};
use std::{
    collections::HashSet,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

// Global flag for graceful shutdown
static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);
static EXIT_HANDLER_STATE: OnceLock<Mutex<bool>> = OnceLock::new();

const POLLING_INTERVAL: Duration = Duration::from_millis(10);

fn inner_setup() -> Result<()> {
    #[cfg(unix)]
    {
        use signal_hook::{
            consts::{SIGHUP, SIGINT, SIGTERM},
            iterator::Signals,
        };
        use std::thread::Builder;

        let mut signals = Signals::new([SIGINT, SIGTERM, SIGHUP]).map_err(|e| {
            crate::error::KeyLoggerError::SignalHandling {
                source: Box::new(e),
            }
        })?;

        Builder::new()
            .name("signal-listener".into())
            .spawn(move || {
                if let Some(sig) = signals.forever().next() {
                    log::info!("Received signal: {sig}");
                    SHOULD_EXIT.store(true, Ordering::Relaxed);
                }
            })
            .expect("spawn signal-listener thread");
    }

    #[cfg(windows)]
    {
        ctrlc::set_handler(|| {
            SHOULD_EXIT.store(true, Ordering::Relaxed);
        })?;
    }

    Ok(())
}

pub fn setup_exit_handler() -> Result<()> {
    let m = EXIT_HANDLER_STATE.get_or_init(|| Mutex::new(false));
    let mut inited = m.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    if *inited {
        return Ok(());
    }

    inner_setup()?;
    *inited = true;
    Ok(())
}

#[inline]
pub fn should_exit() -> bool {
    SHOULD_EXIT.load(Ordering::Relaxed)
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
pub(crate) fn start_key_monitoring(stats: stats::KeyStatistics) -> Result<()> {
    debug!("Starting keyboard monitoring...");
    debug!("Press keys on your keyboard - they will be counted");

    // Initialize device state for keyboard polling
    let device_state = DeviceState::new();
    let mut prev_keys: HashSet<Keycode> = HashSet::with_capacity(16);
    let mut curr_keys: HashSet<Keycode> = HashSet::with_capacity(16);

    let mut buf: Vec<&'static str> = Vec::with_capacity(16);

    loop {
        // Check if we should exit
        if should_exit() {
            break;
        }

        curr_keys.clear();
        curr_keys.extend(device_state.get_keys());

        // Process newly pressed keys
        buf.clear();
        for &keycode in curr_keys.difference(&prev_keys) {
            buf.push(keycode_to_str(keycode));
        }
        if let Err(e) = stats::add_many(&stats, buf.iter().copied()) {
            error!("Failed to record keys: {e}");
        }

        // Update previous state
        std::mem::swap(&mut prev_keys, &mut curr_keys);

        // Small delay to avoid excessive CPU usage
        thread::sleep(POLLING_INTERVAL);
    }

    debug!("Keyboard monitoring stopped");
    Ok(())
}

// Convert Keycode to human-readable string
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
fn keycode_to_str(k: Keycode) -> &'static str {
    match k {
        Keycode::A => "A",
        Keycode::B => "B",
        Keycode::C => "C",
        Keycode::D => "D",
        Keycode::E => "E",
        Keycode::F => "F",
        Keycode::G => "G",
        Keycode::H => "H",
        Keycode::I => "I",
        Keycode::J => "J",
        Keycode::K => "K",
        Keycode::L => "L",
        Keycode::M => "M",
        Keycode::N => "N",
        Keycode::O => "O",
        Keycode::P => "P",
        Keycode::Q => "Q",
        Keycode::R => "R",
        Keycode::S => "S",
        Keycode::T => "T",
        Keycode::U => "U",
        Keycode::V => "V",
        Keycode::W => "W",
        Keycode::X => "X",
        Keycode::Y => "Y",
        Keycode::Z => "Z",
        Keycode::Key0 => "0",
        Keycode::Key1 => "1",
        Keycode::Key2 => "2",
        Keycode::Key3 => "3",
        Keycode::Key4 => "4",
        Keycode::Key5 => "5",
        Keycode::Key6 => "6",
        Keycode::Key7 => "7",
        Keycode::Key8 => "8",
        Keycode::Key9 => "9",
        Keycode::Space => "Space",
        Keycode::Enter => "Enter",
        Keycode::Tab => "Tab",
        Keycode::Backspace => "Backspace",
        Keycode::Delete => "Delete",
        Keycode::Escape => "Escape",
        Keycode::LShift => "LeftShift",
        Keycode::RShift => "RightShift",
        Keycode::LControl => "LeftControl",
        Keycode::RControl => "RightControl",
        Keycode::LAlt | Keycode::LOption => "LeftAlt",
        Keycode::RAlt | Keycode::ROption => "RightAlt",
        Keycode::LMeta | Keycode::Command => "LeftMeta",
        Keycode::RMeta | Keycode::RCommand => "RightMeta",
        Keycode::Up => "ArrowUp",
        Keycode::Down => "ArrowDown",
        Keycode::Left => "ArrowLeft",
        Keycode::Right => "ArrowRight",
        Keycode::Comma => "Comma",
        Keycode::Dot => "Period",
        Keycode::Semicolon => "Semicolon",
        Keycode::Apostrophe => "Apostrophe",
        Keycode::Grave => "Grave",
        Keycode::Minus => "Minus",
        Keycode::Equal => "Equal",
        Keycode::LeftBracket => "LeftBracket",
        Keycode::RightBracket => "RightBracket",
        Keycode::BackSlash => "Backslash",
        Keycode::Slash => "Slash",
        Keycode::F1 => "F1",
        Keycode::F2 => "F2",
        Keycode::F3 => "F3",
        Keycode::F4 => "F4",
        Keycode::F5 => "F5",
        Keycode::F6 => "F6",
        Keycode::F7 => "F7",
        Keycode::F8 => "F8",
        Keycode::F9 => "F9",
        Keycode::F10 => "F10",
        Keycode::F11 => "F11",
        Keycode::F12 => "F12",
        Keycode::F13 => "F13",
        Keycode::F14 => "F14",
        Keycode::F15 => "F15",
        Keycode::F16 => "F16",
        Keycode::F17 => "F17",
        Keycode::F18 => "F18",
        Keycode::F19 => "F19",
        Keycode::F20 => "F20",
        Keycode::Home => "Home",
        Keycode::End => "End",
        Keycode::PageUp => "PageUp",
        Keycode::PageDown => "PageDown",
        Keycode::Insert => "Insert",
        Keycode::CapsLock => "CapsLock",
        Keycode::Numpad0 => "Numpad0",
        Keycode::Numpad1 => "Numpad1",
        Keycode::Numpad2 => "Numpad2",
        Keycode::Numpad3 => "Numpad3",
        Keycode::Numpad4 => "Numpad4",
        Keycode::Numpad5 => "Numpad5",
        Keycode::Numpad6 => "Numpad6",
        Keycode::Numpad7 => "Numpad7",
        Keycode::Numpad8 => "Numpad8",
        Keycode::Numpad9 => "Numpad9",
        Keycode::NumpadAdd => "NumpadAdd",
        Keycode::NumpadSubtract => "NumpadSubtract",
        Keycode::NumpadMultiply => "NumpadMultiply",
        Keycode::NumpadDivide => "NumpadDivide",
        Keycode::NumpadEnter => "NumpadEnter",
        Keycode::NumpadEquals => "NumpadEquals",
        Keycode::NumpadDecimal => "NumpadDecimal",
    }
}
