use std::path::PathBuf;
use std::process::ExitStatus;

use axoupdater::AxoupdateError;
use reqwest::StatusCode;
use thiserror::Error;

use super::data_transfer_objects::release_tag_dto::ReleaseTagDTO;

#[derive(Debug, Error)]
pub enum UpdateError {
    /// No install receipt: the CLI was built with `cargo install`, installed
    /// by a package manager or unpacked from an archive by hand.
    #[error(
        "this copy of zeldoc was not installed with the zeldoc installer, so it cannot update \
         itself; update it the way you installed it, or run the install command from the README"
    )]
    NotInstalledByInstaller,
    /// The installer installed zeldoc elsewhere, and this is another copy.
    #[error(
        "this copy of zeldoc ({}) is not the one the installer put in {}; update it the way you \
         installed it",
        .executable.display(),
        .installer_directory.display()
    )]
    OtherCopyInstalled {
        executable: PathBuf,
        installer_directory: PathBuf,
    },
    #[error("could not read the receipt the zeldoc installer left")]
    Receipt(#[source] Box<AxoupdateError>),
    #[error("could not create the HTTP client")]
    HttpClient(#[source] reqwest::Error),
    #[error("could not ask GitHub for the latest release of zeldoc")]
    LatestRelease(#[source] reqwest::Error),
    /// `/releases/latest` did not redirect to a release.
    #[error("GitHub did not say which release of zeldoc is the latest (it answered {status})")]
    NoLatestRelease { status: StatusCode },
    #[error("the latest release of zeldoc, {tag}, is not tagged with a version")]
    UnversionedRelease { tag: ReleaseTagDTO },
    #[error("release {tag} of zeldoc has no installer for this platform")]
    NoInstaller { tag: ReleaseTagDTO },
    #[error("could not download the zeldoc installer")]
    InstallerDownload(#[source] reqwest::Error),
    #[error("could not save the zeldoc installer in a temporary folder")]
    SaveInstaller(#[source] std::io::Error),
    #[error("could not start the zeldoc installer")]
    InstallerStart(#[source] std::io::Error),
    #[error("the zeldoc installer failed ({status})")]
    InstallerFailed { status: ExitStatus },
    /// Windows only: the running zeldoc.exe has to make way for the new one.
    #[error("could not move the running zeldoc.exe aside for the new version")]
    MoveRunningCopyAside(#[source] std::io::Error),
    /// Windows only: the installer failed and zeldoc.exe is still renamed.
    #[error(
        "the zeldoc installer failed, and zeldoc.exe could not be put back; rename {} to \
         zeldoc.exe",
        .moved.display()
    )]
    RestoreRunningCopy {
        moved: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl From<AxoupdateError> for UpdateError {
    fn from(error: AxoupdateError) -> Self {
        match error {
            AxoupdateError::NoReceipt { .. } => Self::NotInstalledByInstaller,
            error => Self::Receipt(Box::new(error)),
        }
    }
}
