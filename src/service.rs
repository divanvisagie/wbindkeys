use std::fs;
use std::path::{Path, PathBuf};

use crate::permissions::{is_root, run};

const ISSUES_URL: &str = "https://github.com/divanvisagie/wbindkeys/issues";

/// The service managers wbindkeys knows how to install a per-user service with.
enum ServiceManager {
    Systemd,
}

/// Works out which service manager this system uses, or explains that it
/// isn't supported yet.
fn detect() -> Result<ServiceManager, String> {
    match std::env::consts::OS {
        // The same check as sd_booted(3): only present when systemd is PID 1.
        "linux" if Path::new("/run/systemd/system").is_dir() => Ok(ServiceManager::Systemd),
        "linux" => Err(unsupported("Linux without systemd")),
        os => Err(unsupported(os_name(os))),
    }
}

fn os_name(os: &str) -> &str {
    match os {
        "freebsd" => "FreeBSD",
        "openbsd" => "OpenBSD",
        "netbsd" => "NetBSD",
        "dragonfly" => "DragonFly BSD",
        "macos" => "macOS",
        other => other,
    }
}

fn unsupported(system: &str) -> String {
    format!(
        "installing wbindkeys as a service on {} is not supported yet, \
         please send a request to the author at {}",
        system, ISSUES_URL
    )
}

/// The service runs as whoever installs it, so refuse to install it for root.
fn ensure_not_root() -> Result<(), String> {
    if is_root() {
        return Err("run this as your normal user, not with sudo; \
                    the service runs as the user who installs it"
            .to_string());
    }
    Ok(())
}

/// Installs wbindkeys as a service that starts when the current user logs
/// in, running the binary this was invoked as, and (re)starts it now.
pub fn install() -> Result<(), String> {
    let manager = detect()?;
    ensure_not_root()?;
    let exe = std::env::current_exe().map_err(|err| format!("locating the wbindkeys binary: {}", err))?;

    match manager {
        ServiceManager::Systemd => systemd::install(&exe),
    }
}

/// Stops the service and removes what `install` set up.
pub fn uninstall() -> Result<(), String> {
    let manager = detect()?;
    ensure_not_root()?;

    match manager {
        ServiceManager::Systemd => systemd::uninstall(),
    }
}

mod systemd {
    use super::*;

    const UNIT: &str = "wbindkeys.service";

    /// $XDG_CONFIG_HOME/systemd/user/wbindkeys.service
    fn unit_path() -> Result<PathBuf, String> {
        let config = dirs::config_dir().ok_or("could not find your config directory")?;
        Ok(config.join("systemd/user").join(UNIT))
    }

    pub(super) fn unit_file(exe: &Path) -> String {
        // Quote the path in case it has spaces, and escape `%`, which systemd
        // treats as the start of a specifier.
        let exec = exe.display().to_string().replace('%', "%%");
        format!(
            "[Unit]\n\
             Description=wbindkeys service\n\
             \n\
             [Service]\n\
             Type=simple\n\
             ExecStart=\"{}\"\n\
             \n\
             [Install]\n\
             WantedBy=default.target\n",
            exec
        )
    }

    fn systemctl(args: &[&str]) -> Result<(), String> {
        let mut all = vec!["--user"];
        all.extend_from_slice(args);
        run("systemctl", &all)
    }

    pub(super) fn install(exe: &Path) -> Result<(), String> {
        let path = unit_path()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|err| format!("creating {}: {}", dir.display(), err))?;
        }
        fs::write(&path, unit_file(exe)).map_err(|err| format!("writing {}: {}", path.display(), err))?;
        println!("Installed {}", path.display());

        systemctl(&["daemon-reload"])?;
        systemctl(&["enable", UNIT])?;
        // restart rather than start, so reinstalling picks up a new binary
        systemctl(&["restart", UNIT])?;
        println!("Enabled and started {}", UNIT);
        Ok(())
    }

    pub(super) fn uninstall() -> Result<(), String> {
        let path = unit_path()?;
        if !path.exists() {
            println!("{} is not installed, nothing to do", path.display());
            return Ok(());
        }

        systemctl(&["disable", "--now", UNIT])?;
        fs::remove_file(&path).map_err(|err| format!("removing {}: {}", path.display(), err))?;
        systemctl(&["daemon-reload"])?;
        println!("Stopped, disabled and removed {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_unit_runs_given_binary() {
        let unit = systemd::unit_file(Path::new("/home/me/my bin/50%/wbindkeys"));
        assert!(unit.contains("ExecStart=\"/home/me/my bin/50%%/wbindkeys\"\n"), "{}", unit);
        assert!(unit.contains("WantedBy=default.target"), "{}", unit);
    }

    #[test]
    fn unsupported_message_names_system_and_where_to_ask() {
        let message = unsupported(os_name("freebsd"));
        assert!(message.contains("FreeBSD is not supported yet"), "{}", message);
        assert!(message.contains(ISSUES_URL), "{}", message);
    }
}
