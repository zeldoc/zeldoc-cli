//! The "newer release available" line printed after a command.
//!
//! At most once a day, a background thread asks GitHub for the latest release
//! while the command runs, and the command waits for it at most a second; the
//! answer is cached in the platform cache directory. The line goes to stderr, and only when stdout and stderr are both
//! terminals: scripts, agents and `$(zeldoc auth token)` in a shell profile
//! never see it or wait for it. Copies that `zeldoc update` cannot replace are
//! never checked.

use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use axoupdater::Version;
use chrono::{DateTime, TimeDelta, Utc};

use super::data_transfer_objects::update_check_dto::UpdateCheckDTO;
use super::update_error::UpdateError;
use super::update_outcome::UpdateOutcome;
use super::update_outcome_text;
use super::update_service::{UpdateService, UpdateServiceImpl};
use crate::constants::environment;

const FILE_NAME: &str = "update-check.json";
const CHECK_INTERVAL: TimeDelta = TimeDelta::hours(24);
/// Limits each request of the background check.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
/// How long a finished command waits for a check that is still running. A
/// check that misses it counts as failed, so at most one command a day waits.
const WAIT_AFTER_COMMAND: Duration = Duration::from_secs(1);

pub struct UpdateNotice {
    path: PathBuf,
    current: Version,
    previous: Option<UpdateCheckDTO>,
    /// The background check, when one was due.
    check: Option<Receiver<Result<UpdateOutcome, UpdateError>>>,
}

impl UpdateNotice {
    /// Starts a check in the background when one is due; `None` when no
    /// notice should be shown.
    pub fn start() -> Option<Self> {
        let turned_off =
            env::var_os(environment::NO_UPDATE_CHECK).is_some_and(|value| !value.is_empty());
        if turned_off || !io::stdout().is_terminal() || !io::stderr().is_terminal() {
            return None;
        }
        let service = UpdateServiceImpl::with_timeout(REQUEST_TIMEOUT);
        if !service.can_update() {
            return None;
        }
        let path = dirs::cache_dir()?.join("zeldoc").join(FILE_NAME);
        let previous = read_check(&path);
        let check = is_check_due(previous.as_ref(), Utc::now()).then(|| {
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || {
                // The receiver is gone when the command finished first.
                let _ = sender.send(service.check());
            });
            receiver
        });
        Some(Self {
            path,
            current: UpdateServiceImpl::new().current_version().clone(),
            previous,
            check,
        })
    }

    /// Writes the notice to `output` when a newer release than this one is
    /// known. Never fails: the notice is a courtesy.
    pub fn finish(self, output: &mut impl Write) {
        let mut latest = self
            .previous
            .as_ref()
            .and_then(|check| check.latest_version.clone());
        if let Some(check) = self.check {
            // A failed or slow check (offline, GitHub down) also waits a day,
            // so commands are not held up again and GitHub is not asked
            // again after every command.
            if let Ok(Ok(outcome)) = check.recv_timeout(WAIT_AFTER_COMMAND) {
                latest = Some(outcome.latest_version().clone());
            }
            let _ = write_check(
                &self.path,
                &UpdateCheckDTO {
                    checked_at: Utc::now(),
                    latest_version: latest.clone(),
                },
            );
        }
        if let Some(latest) = latest.filter(|latest| *latest > self.current) {
            let outcome = UpdateOutcome::Available {
                current: self.current,
                latest,
            };
            let _ = update_outcome_text::write_outcome(output, &outcome);
        }
    }
}

fn is_check_due(previous: Option<&UpdateCheckDTO>, now: DateTime<Utc>) -> bool {
    previous.is_none_or(|check| now - check.checked_at >= CHECK_INTERVAL)
}

/// The cached check; none when it is missing or unreadable.
fn read_check(path: &Path) -> Option<UpdateCheckDTO> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

fn write_check(path: &Path, check: &UpdateCheckDTO) -> io::Result<()> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    fs::write(path, serde_json::to_vec(check)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checked_at(time: &str) -> UpdateCheckDTO {
        UpdateCheckDTO {
            checked_at: time.parse().unwrap(),
            latest_version: None,
        }
    }

    #[test]
    fn checks_once_a_day() {
        let now = "2026-10-06T12:00:00Z".parse().unwrap();
        assert!(is_check_due(None, now));
        assert!(!is_check_due(
            Some(&checked_at("2026-10-06T01:00:00Z")),
            now
        ));
        assert!(is_check_due(Some(&checked_at("2026-10-05T12:00:00Z")), now));
    }

    #[test]
    fn cached_check_round_trips_and_bad_files_are_ignored() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("zeldoc").join(FILE_NAME);
        assert_eq!(read_check(&path), None);

        let check = UpdateCheckDTO {
            checked_at: "2026-10-06T12:00:00Z".parse().unwrap(),
            latest_version: Some("0.3.0".parse().unwrap()),
        };
        write_check(&path, &check).unwrap();
        assert_eq!(read_check(&path), Some(check));

        fs::write(&path, "not json").unwrap();
        assert_eq!(read_check(&path), None);
    }
}
