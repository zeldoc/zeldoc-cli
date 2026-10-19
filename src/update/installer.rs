//! Runs a release's installer over the installed copy, as the installer
//! would run from the install command in the README.

use std::path::Path;
use std::process::{Command, Stdio};

use super::update_error::UpdateError;

/// Runs the installer at `installer` so it installs into `install_directory`,
/// the folder the first install used. Its progress shows on stderr; its
/// stdout is dropped, as `zeldoc update` reports the outcome there itself.
pub fn run(installer: &Path, install_directory: &Path) -> Result<(), UpdateError> {
    let mut command = installer_command(installer);
    command
        // The installers install to exactly this folder; the second name is
        // the older one, kept for installers that predate the first.
        .env("ZELDOC_INSTALL_DIR", install_directory)
        .env("CARGO_DIST_FORCE_INSTALL_DIR", install_directory)
        // PATH and shell profiles stay as the first install left them.
        .env("ZELDOC_NO_MODIFY_PATH", "1")
        .stdout(Stdio::null());

    #[cfg(windows)]
    let moved_aside = windows::move_running_copy_aside()?;
    let status = command.status();
    #[cfg(windows)]
    windows::finish(
        moved_aside,
        status.as_ref().is_ok_and(|status| status.success()),
    )?;

    let status = status.map_err(UpdateError::InstallerStart)?;
    if !status.success() {
        return Err(UpdateError::InstallerFailed { status });
    }
    Ok(())
}

#[cfg(not(windows))]
fn installer_command(installer: &Path) -> Command {
    // Through `sh` rather than run directly, so a temporary folder mounted
    // without execute permission does not stop it.
    let mut command = Command::new("sh");
    command.arg(installer);
    command
}

#[cfg(windows)]
fn installer_command(installer: &Path) -> Command {
    let mut command = Command::new("powershell");
    command
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(installer)
        // Started from PowerShell 7, Windows PowerShell would otherwise load
        // PowerShell 7's modules and fail.
        .env_remove("PSModulePath");
    command
}

/// Windows does not let the installer overwrite a running executable, but it
/// can be renamed: the running zeldoc.exe steps aside for the new one, and is
/// put back when the installer fails.
#[cfg(windows)]
mod windows {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    use super::UpdateError;

    /// The running executable's own path and the one it was moved to.
    pub type MovedAside = (PathBuf, PathBuf);

    pub fn move_running_copy_aside() -> Result<MovedAside, UpdateError> {
        let running = env::current_exe().map_err(UpdateError::MoveRunningCopyAside)?;
        let mut moved = running.clone().into_os_string();
        moved.push(".previous.exe");
        let moved = PathBuf::from(moved);
        fs::rename(&running, &moved).map_err(UpdateError::MoveRunningCopyAside)?;
        Ok((running, moved))
    }

    pub fn finish((running, moved): MovedAside, installed: bool) -> Result<(), UpdateError> {
        if installed {
            // Deleted once zeldoc exits. The new version is installed either
            // way, so a file left behind does not fail the update.
            let _ = self_replace::self_delete_at(&moved);
            Ok(())
        } else {
            fs::rename(&moved, &running)
                .map_err(|source| UpdateError::RestoreRunningCopy { moved, source })
        }
    }
}
