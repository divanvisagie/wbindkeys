use clap::{Args as ClapArgs, Parser, Subcommand};
use dirs::config_dir;
use input::event::keyboard::KeyState;
use input::{Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use script_manager::ScriptManager;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracker::Tracker;

/// Set by the `--debug` flag; enables `debug!` log output.
pub static DEBUG: AtomicBool = AtomicBool::new(false);

/// Prints to stderr when running with `--debug`.
macro_rules! debug {
    ($($arg:tt)*) => {
        if crate::DEBUG.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("[debug] {}", format!($($arg)*));
        }
    };
}

mod parser;
mod permissions;
mod record;
mod script_manager;
mod service;
mod tracker;

struct WBindKeysInterface;

impl LibinputInterface for WBindKeysInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        let file = OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) || (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) || (flags & O_RDWR != 0))
            .open(path);

        match file {
            Ok(f) => {
                debug!("Opened input device {:?}", path);
                Ok(f.into())
            }
            Err(err) => {
                debug!("Failed to open input device {:?}: {}", path, err);
                Err(err.raw_os_error().unwrap_or(-1))
            }
        }
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

/// Opens every input device on seat0 through libinput.
fn open_input() -> Libinput {
    let mut input = Libinput::new_with_udev(WBindKeysInterface);
    input.udev_assign_seat("seat0").unwrap();
    input
}

/// $XDG_CONFIG_HOME/wbindkeys/init.lua
fn config_path() -> PathBuf {
    config_dir()
        .expect("Failed to load config directory.")
        .join("wbindkeys")
        .join("init.lua")
}

/// A script manager with the user's config loaded, or an error if the config
/// can't be read or run.
fn load_config(config_path: &Path) -> Result<ScriptManager, String> {
    let script_manager = ScriptManager::new();
    script_manager.register_functions().map_err(|err| err.to_string())?;
    debug!("Loading config from {:?}", config_path);
    let script = std::fs::read_to_string(config_path)
        .map_err(|err| format!("reading {}: {}", config_path.display(), err))?;
    script_manager
        .load_script(&script)
        .map_err(|err| format!("running {}: {}", config_path.display(), err))?;
    Ok(script_manager)
}

/// Bind keys, mouse buttons and scroll events to commands on Wayland.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Print log output (input devices, config loading, bindings, key events)
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check or set up permission to read input devices
    Permissions(PermissionsArgs),
    /// Install or remove wbindkeys as a service that starts when you log in
    Service(ServiceArgs),
    /// Record what wbindkeys sees while you press keys, to attach to a bug report
    Record {
        /// File to write the recording to
        #[arg(short, long, default_value = "wbindkeys-recording.txt")]
        output: PathBuf,
    },
    /// Print the wbindkeys(1) manual page
    Man {
        /// Write it under ../share/man/man1 next to the wbindkeys binary
        /// (e.g. ~/.cargo/share/man/man1 after `cargo install`) instead
        #[arg(long)]
        install: bool,
    },
}

#[derive(ClapArgs)]
#[group(required = true, multiple = false)]
struct ServiceArgs {
    /// Install, enable and start the service for the current user
    #[arg(long)]
    install: bool,

    /// Stop, disable and remove the service
    #[arg(long)]
    uninstall: bool,
}

const MAN_PAGE: &str = include_str!("../man/wbindkeys.1");

#[derive(ClapArgs)]
#[group(required = true, multiple = false)]
struct PermissionsArgs {
    /// Check the udev rule is installed and input devices are readable
    #[arg(long)]
    check: bool,

    /// Install the udev rule and re-trigger input devices (needs root)
    #[arg(long)]
    set: bool,
}

fn man(install: bool) -> Result<(), String> {
    if !install {
        std::io::stdout().write_all(MAN_PAGE.as_bytes()).map_err(|err| err.to_string())?;
        return Ok(());
    }

    let exe = std::env::current_exe().map_err(|err| format!("locating the wbindkeys binary: {}", err))?;
    let prefix = exe
        .parent()
        .and_then(|bin| bin.parent())
        .ok_or("the wbindkeys binary has no parent directory to install under")?;
    let dir = prefix.join("share/man/man1");
    let path = dir.join("wbindkeys.1");
    std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(&path, MAN_PAGE))
        .map_err(|err| {
            format!(
                "writing {}: {} (run `wbindkeys man > <path>` to put it somewhere else)",
                path.display(),
                err
            )
        })?;
    println!("installed {}", path.display());
    Ok(())
}

fn main() {
    let args = Args::parse();
    DEBUG.store(args.debug, Ordering::Relaxed);

    match args.command {
        Some(Commands::Permissions(perms)) => {
            let ok = if perms.set {
                permissions::set().map_err(|err| eprintln!("error: {}", err)).is_ok()
            } else {
                permissions::check()
            };
            std::process::exit(if ok { 0 } else { 1 });
        }
        Some(Commands::Service(service)) => {
            let result = if service.install { service::install() } else { service::uninstall() };
            let ok = result.map_err(|err| eprintln!("error: {}", err)).is_ok();
            std::process::exit(if ok { 0 } else { 1 });
        }
        Some(Commands::Record { output }) => {
            let ok = record::record(&output).map_err(|err| eprintln!("error: {}", err)).is_ok();
            std::process::exit(if ok { 0 } else { 1 });
        }
        Some(Commands::Man { install }) => {
            let ok = man(install).map_err(|err| eprintln!("error: {}", err)).is_ok();
            std::process::exit(if ok { 0 } else { 1 });
        }
        None => {}
    }

    let mut input = open_input();

    let config_path = config_path();
    if !config_path.exists() {
        panic!("Config file not found at {:?}", config_path);
    }
    let script_manager = load_config(&config_path).unwrap();
    debug!("Config loaded, listening for input events");

    let mut tracker = Tracker::new();
    loop {
        input.dispatch().unwrap();

        for event in &mut input {
            if let Some(combo) = tracker.handle(&event, Instant::now()).and_then(|observed| observed.combo) {
                script_manager.handle_action(combo, KeyState::Pressed);
            }
        }
        tracker.expire_scrolls(Instant::now());

        // small sleep to avoid busy loop (libinput often blocks anyway)
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// The man page is written by hand, so make sure it keeps up with the
    /// CLI: every subcommand and long flag needs at least a mention.
    #[test]
    fn man_page_covers_cli() {
        fn check(cmd: &clap::Command, missing: &mut Vec<String>) {
            for arg in cmd.get_arguments() {
                // mdoc writes `--flag` as `Fl -flag`.
                if let Some(long) = arg.get_long().filter(|long| !MAN_PAGE.contains(&format!("Fl -{long}"))) {
                    missing.push(format!("--{long}"));
                }
            }
            for sub in cmd.get_subcommands() {
                if sub.get_name() != "help" && !MAN_PAGE.contains(&format!("Cm {}", sub.get_name())) {
                    missing.push(sub.get_name().to_string());
                }
                check(sub, missing);
            }
        }

        let mut missing = Vec::new();
        check(&Args::command(), &mut missing);
        assert!(missing.is_empty(), "not documented in man/wbindkeys.1: {}", missing.join(", "));
    }
}
