/// Addresses and paths of the Zeldoc.ai API.
pub mod api {
    /// The gateway's OpenAI-compatible API. Deliberately not configurable: the
    /// CLI sends the API key to it, and a variable that redirected it would let
    /// anything that can set the environment collect the key.
    pub const BASE_URL: &str = "https://api.zeldoc.ai/v1";
    /// The model catalog. Unlike `/v1/models`, which only lists names, it has
    /// limits, prices and capabilities.
    pub const MODEL_CATALOG_PATH: &str = "/zeldoc/models";
    pub const SEARCH_PATH: &str = "/search/zeldoc-search";
    /// What the API key itself used over a period.
    pub const USAGE_PATH: &str = "/zeldoc/usage";
    /// The API key's own name and key field values.
    pub const KEY_DETAILS_PATH: &str = "/zeldoc/key";
    pub const API_KEY_DOCS_URL: &str =
        "https://docs.zeldoc.ai/connect-opencode#generate-an-api-key";
}

/// Where releases of the CLI are published.
pub mod releases {
    /// The release pages on github.com. Not GitHub's API: without a GitHub
    /// login it allows 60 requests an hour per IP address, and everyone on an
    /// office network shares one, so other tools there often use it up.
    /// Deliberately not configurable: `zeldoc update` runs the installer it
    /// downloads from here.
    pub const PAGES_URL: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/releases");
}

/// Environment variables the CLI reads.
pub mod environment {
    /// An API key; used when neither ZELDOC_PROFILE, `--profile` nor a pin
    /// file picks a saved profile.
    pub const API_KEY: &str = "ZELDOC_API_KEY";
    /// The saved profile to use, like `--profile`.
    pub const PROFILE: &str = "ZELDOC_PROFILE";
    /// Overrides where the saved key lives.
    pub const CONFIG_DIRECTORY: &str = "ZELDOC_CONFIG_DIR";
    /// Any non-empty value turns off the daily check for a newer release.
    pub const NO_UPDATE_CHECK: &str = "ZELDOC_NO_UPDATE_CHECK";
}

/// Files the CLI reads outside its config directory.
pub mod files {
    /// Pins a folder and the folders below it to a saved profile.
    pub const PROFILE_PIN: &str = ".zeldoc-profile";
}
