use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/69-wbindkeys.rules";
const UDEV_RULE: &str =
    r#"ACTION=="add", KERNEL=="event*", SUBSYSTEM=="input", TAG+="uaccess", TAG+="seat""#;

pub(crate) fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Returns every /dev/input/event* device, sorted by event number.
fn input_devices() -> std::io::Result<Vec<PathBuf>> {
    let mut devices: Vec<PathBuf> = fs::read_dir("/dev/input")?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("event"))
        })
        .collect();
    devices.sort_by_key(|path| {
        path.file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.trim_start_matches("event").parse::<u32>().ok())
            .unwrap_or(u32::MAX)
    });
    Ok(devices)
}

/// Human readable device name from sysfs, e.g. "AT Translated Set 2 keyboard".
fn device_name(device: &Path) -> String {
    let event = device.file_name().and_then(|n| n.to_str()).unwrap_or("");
    fs::read_to_string(format!("/sys/class/input/{}/device/name", event))
        .map(|name| name.trim().to_string())
        .unwrap_or_else(|_| "unknown device".to_string())
}

/// Checks the udev rule is installed and every input device can be opened by
/// the current user. Returns true if everything is in order.
pub fn check() -> bool {
    let mut ok = true;

    match fs::read_to_string(UDEV_RULE_PATH) {
        Ok(rule) if rule.trim() == UDEV_RULE => {
            println!("[ok]   udev rule installed at {}", UDEV_RULE_PATH);
        }
        Ok(_) => {
            println!("[fail] udev rule at {} is out of date", UDEV_RULE_PATH);
            ok = false;
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            println!("[fail] udev rule not found at {}", UDEV_RULE_PATH);
            ok = false;
        }
        Err(err) => {
            println!("[fail] could not read udev rule at {}: {}", UDEV_RULE_PATH, err);
            ok = false;
        }
    }

    let devices = match input_devices() {
        Ok(devices) => devices,
        Err(err) => {
            println!("[fail] could not list /dev/input: {}", err);
            return false;
        }
    };

    let denied: Vec<(&PathBuf, std::io::Error)> = devices
        .iter()
        .filter_map(|device| {
            OpenOptions::new()
                .read(true)
                .open(device)
                .err()
                .map(|err| (device, err))
        })
        .collect();

    if devices.is_empty() {
        println!("[fail] no input devices found in /dev/input");
        ok = false;
    } else if denied.is_empty() {
        println!("[ok]   all {} input devices are readable", devices.len());
    } else {
        println!(
            "[fail] {} of {} input devices are not readable:",
            denied.len(),
            devices.len()
        );
        for (device, err) in &denied {
            println!("         {} ({}): {}", device.display(), device_name(device), err);
        }
        ok = false;
    }

    if is_root() {
        println!();
        println!("Note: running as root, so device access reflects root, not your user.");
        println!("Run this check without sudo to see what wbindkeys will have access to.");
    }

    if !ok {
        println!();
        println!("To fix, run:");
        println!("  {}", set_command_hint());
        println!("If devices are still not readable afterwards, log out and back in.");
    }

    ok
}

/// Installs the udev rule and re-triggers input devices so the current seat
/// user gets access. Must be run as root.
pub fn set() -> Result<(), String> {
    if !is_root() {
        return Err(format!(
            "setting permissions requires root, run:\n  {}",
            set_command_hint()
        ));
    }

    fs::write(UDEV_RULE_PATH, format!("{}\n", UDEV_RULE))
        .map_err(|err| format!("failed to write {}: {}", UDEV_RULE_PATH, err))?;
    println!("Installed udev rule at {}", UDEV_RULE_PATH);

    run("udevadm", &["control", "--reload-rules"])?;
    run(
        "udevadm",
        &["trigger", "--action=add", "--subsystem-match=input"],
    )?;
    run("udevadm", &["settle"])?;
    println!("Reloaded udev rules and re-triggered input devices");
    println!();
    println!("Run `wbindkeys permissions --check` (without sudo) to verify.");

    Ok(())
}

pub(crate) fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|err| format!("failed to run {}: {}", program, err))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{} {}` failed with {}", program, args.join(" "), status))
    }
}

/// `sudo <full path to this binary> permissions --set`. Uses the full path
/// because sudo's secure_path usually doesn't include ~/.local/bin.
fn set_command_hint() -> String {
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "wbindkeys".to_string());
    format!("sudo {} permissions --set", exe)
}
