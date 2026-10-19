use std::env;
use std::path::PathBuf;
use std::time::Duration;

use axoupdater::{AxoUpdater, Version};
use reqwest::blocking::Client;
use reqwest::redirect;

use super::data_transfer_objects::release_tag_dto::ReleaseTagDTO;
use super::installer;
use super::release_downloads;
use super::update_error::UpdateError;
use super::update_outcome::UpdateOutcome;

/// The name the installer records its install receipt under.
const APP_NAME: &str = "zeldoc";

pub trait UpdateService {
    /// The running version.
    fn current_version(&self) -> &Version;
    /// Whether this executable is the copy the installer installed, the only
    /// one `update` can replace. Reads local files only.
    fn can_update(&self) -> bool;
    /// Whether a newer release than the running version exists.
    fn check(&self) -> Result<UpdateOutcome, UpdateError>;
    /// Installs the latest release in place of the running version, when it
    /// is newer.
    fn update(&self) -> Result<UpdateOutcome, UpdateError>;
}

/// Updates through the release's own shell or PowerShell installer, run over
/// the folder named in the receipt the installer wrote when zeldoc was first
/// installed. Everything comes from the release pages on github.com, never
/// GitHub's API.
pub struct UpdateServiceImpl {
    current: Version,
    /// Limits each request to GitHub; none when the user waits for an update.
    timeout: Option<Duration>,
}

impl UpdateServiceImpl {
    pub fn new() -> Self {
        Self {
            current: env!("CARGO_PKG_VERSION")
                .parse()
                .expect("the crate version is a valid version"),
            timeout: None,
        }
    }

    /// For checks in the background, which must not hold a command up.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            ..Self::new()
        }
    }

    /// The folder the installer installed this executable into, or an error
    /// when the installer did not install it.
    fn install_directory(&self) -> Result<PathBuf, UpdateError> {
        let mut updater = AxoUpdater::new_for(APP_NAME);
        updater.load_receipt()?;
        let install_directory = updater.install_prefix_root()?.into_std_path_buf();
        if !updater.check_receipt_is_for_this_executable()? {
            return Err(UpdateError::OtherCopyInstalled {
                executable: env::current_exe().unwrap_or_default(),
                installer_directory: install_directory,
            });
        }
        Ok(install_directory)
    }

    /// The latest release and its version.
    fn latest_release(&self) -> Result<(ReleaseTagDTO, Version), UpdateError> {
        let tag = release_downloads::latest_release_tag(&self.client(redirect::Policy::none())?)?;
        let version = tag
            .version()
            .ok_or_else(|| UpdateError::UnversionedRelease { tag: tag.clone() })?;
        Ok((tag, version))
    }

    fn client(&self, redirects: redirect::Policy) -> Result<Client, UpdateError> {
        Client::builder()
            .user_agent(concat!("zeldoc-cli/", env!("CARGO_PKG_VERSION")))
            .timeout(self.timeout)
            .redirect(redirects)
            .build()
            .map_err(UpdateError::HttpClient)
    }
}

impl Default for UpdateServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateService for UpdateServiceImpl {
    fn current_version(&self) -> &Version {
        &self.current
    }

    fn can_update(&self) -> bool {
        self.install_directory().is_ok()
    }

    fn check(&self) -> Result<UpdateOutcome, UpdateError> {
        self.install_directory()?;
        let (_, latest) = self.latest_release()?;
        Ok(if latest > self.current {
            UpdateOutcome::Available {
                current: self.current.clone(),
                latest,
            }
        } else {
            UpdateOutcome::UpToDate {
                current: self.current.clone(),
            }
        })
    }

    fn update(&self) -> Result<UpdateOutcome, UpdateError> {
        let install_directory = self.install_directory()?;
        let (tag, latest) = self.latest_release()?;
        if latest <= self.current {
            return Ok(UpdateOutcome::UpToDate {
                current: self.current.clone(),
            });
        }
        // The installer's download goes through GitHub's redirect to where
        // release files are stored.
        let client = self.client(redirect::Policy::default())?;
        let directory = tempfile::tempdir().map_err(UpdateError::SaveInstaller)?;
        let installer = release_downloads::download_installer(&client, &tag, directory.path())?;
        installer::run(&installer, &install_directory)?;
        Ok(UpdateOutcome::Updated {
            previous: self.current.clone(),
            installed: latest,
        })
    }
}
