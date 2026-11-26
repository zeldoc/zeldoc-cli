use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

use crate::auth::auth_command::AuthCommand;
use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;
use crate::models::models_command::ModelsCommand;
use crate::search::search_command::SearchCommand;
use crate::update::update_command::UpdateCommand;
use crate::usage::usage_command::UsageCommand;

// Help text lives on the variants: clap shows it for `zeldoc <command> --help`.
#[derive(Subcommand)]
pub enum Command {
    /// Save, switch, check or remove the API keys the CLI uses
    #[command(subcommand)]
    Auth(AuthCommand),
    /// List the models your API key can use
    Models(ModelsCommand),
    /// Search the web through Zeldoc.ai's search endpoint
    ///
    /// Queries go to Zeldoc.ai's own SearXNG instance with your API key.
    /// General searches use Staan, an EU-based provider; `--categories it` and
    /// `--engines` can reach GitHub, Stack Overflow and other services outside
    /// the EU. Keep personal and confidential information out of queries.
    /// Searches count towards your key's rate limit but are not billed.
    Search(SearchCommand),
    /// Show what your API key has used: requests, tokens and cost per model
    ///
    /// Reports only the key the CLI uses, not other keys of your
    /// organization; the dashboard at app.zeldoc.ai has the whole
    /// organization. Periods are in UTC. Also shows the key's monthly spend
    /// limit, the organization's prepaid credits and your ZDev allowance,
    /// when there are any. Costs are in USD, as the dashboard shows them;
    /// Zeldoc's own models are covered by the subscription and cost 0.
    /// Requests can take a minute to show up.
    Usage(UsageCommand),
    /// Update zeldoc to the latest release
    ///
    /// Downloads the latest release's installer from GitHub and runs it, so the
    /// new version replaces this one in the same folder. Works for copies
    /// installed with the shell or PowerShell installer; a copy built with
    /// `cargo install` or unpacked from a release archive by hand has to be
    /// updated the same way it was installed. Exits with status 1 when the
    /// update fails, and 0 when zeldoc is already the latest version.
    ///
    /// Other commands check for a newer release once a day and say so on
    /// stderr, but only when run in a terminal; ZELDOC_NO_UPDATE_CHECK=1
    /// turns that off.
    Update(UpdateCommand),
}

impl Command {
    /// `zeldoc update` reports on new releases itself.
    pub fn is_update(&self) -> bool {
        matches!(self, Self::Update(_))
    }

    /// Run the command; `profile` is the profile asked for with `--profile`
    /// or ZELDOC_PROFILE.
    pub fn run(self, profile: Option<&ProfileNameDTO>) -> Result<ExitCode> {
        match self {
            Self::Auth(command) => command.run(profile),
            Self::Models(command) => command.run(profile),
            Self::Search(command) => command.run(profile),
            Self::Usage(command) => command.run(profile),
            Self::Update(command) => command.run(),
        }
    }
}
