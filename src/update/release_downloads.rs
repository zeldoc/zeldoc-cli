//! The latest release and its installer, from the release pages on
//! github.com rather than GitHub's API (see `releases::PAGES_URL`). The
//! installer then downloads the release archive from there too.

use std::fs;
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use reqwest::header::LOCATION;
use reqwest::{StatusCode, Url};

use super::data_transfer_objects::release_tag_dto::ReleaseTagDTO;
use super::update_error::UpdateError;
use crate::constants::releases;

/// The shell installer, or the PowerShell one on Windows.
const INSTALLER_FILE_NAME: &str = if cfg!(windows) {
    "zeldoc-installer.ps1"
} else {
    "zeldoc-installer.sh"
};

/// The latest release's tag. GitHub answers `/releases/latest` with a
/// redirect to the release's page, `/releases/tag/<tag>`; `client` must not
/// follow redirects, so the tag can be read from it.
pub fn latest_release_tag(client: &Client) -> Result<ReleaseTagDTO, UpdateError> {
    let response = client
        .get(format!("{}/latest", releases::PAGES_URL))
        .send()
        .map_err(UpdateError::LatestRelease)?;
    let status = response.status();
    let release_page = response
        .headers()
        .get(LOCATION)
        .and_then(|location| location.to_str().ok())
        .and_then(|location| response.url().join(location).ok());
    release_page
        .filter(|_| status.is_redirection())
        .and_then(|release_page| tag_of_release_page(&release_page))
        .ok_or(UpdateError::NoLatestRelease { status })
}

/// Downloads the installer of the release `tag` into `directory`, for this
/// platform.
pub fn download_installer(
    client: &Client,
    tag: &ReleaseTagDTO,
    directory: &Path,
) -> Result<PathBuf, UpdateError> {
    let response = client
        .get(installer_url(tag))
        .send()
        .map_err(UpdateError::InstallerDownload)?;
    if response.status() == StatusCode::NOT_FOUND {
        return Err(UpdateError::NoInstaller { tag: tag.clone() });
    }
    let installer = response
        .error_for_status()
        .and_then(|response| response.bytes())
        .map_err(UpdateError::InstallerDownload)?;
    let path = directory.join(INSTALLER_FILE_NAME);
    fs::write(&path, installer).map_err(UpdateError::SaveInstaller)?;
    Ok(path)
}

/// `v0.4.0` from `https://github.com/<owner>/<repository>/releases/tag/v0.4.0`.
fn tag_of_release_page(release_page: &Url) -> Option<ReleaseTagDTO> {
    let mut segments = release_page.path_segments()?.rev();
    let tag = segments.next().filter(|tag| !tag.is_empty())?;
    (segments.next() == Some("tag") && segments.next() == Some("releases"))
        .then(|| ReleaseTagDTO::from(tag.to_owned()))
}

fn installer_url(tag: &ReleaseTagDTO) -> String {
    format!(
        "{}/download/{tag}/{INSTALLER_FILE_NAME}",
        releases::PAGES_URL
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag_of(release_page: &str) -> Option<ReleaseTagDTO> {
        tag_of_release_page(&release_page.parse().unwrap())
    }

    #[test]
    fn reads_the_tag_from_the_release_page() {
        assert_eq!(
            tag_of("https://github.com/zeldoc/zeldoc-cli/releases/tag/v0.4.0"),
            Some(ReleaseTagDTO::from("v0.4.0".to_owned()))
        );
    }

    #[test]
    fn other_pages_have_no_tag() {
        // Where GitHub sends `/releases/latest` while there is no release.
        assert_eq!(
            tag_of("https://github.com/zeldoc/zeldoc-cli/releases"),
            None
        );
        assert_eq!(
            tag_of("https://github.com/zeldoc/zeldoc-cli/releases/tag/"),
            None
        );
        assert_eq!(tag_of("https://github.com/login"), None);
    }

    #[test]
    fn installer_is_the_release_asset_for_this_platform() {
        let url = installer_url(&ReleaseTagDTO::from("v0.4.0".to_owned()));
        let expected = if cfg!(windows) {
            "https://github.com/zeldoc/zeldoc-cli/releases/download/v0.4.0/zeldoc-installer.ps1"
        } else {
            "https://github.com/zeldoc/zeldoc-cli/releases/download/v0.4.0/zeldoc-installer.sh"
        };
        assert_eq!(url, expected);
    }
}
