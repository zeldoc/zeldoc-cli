# Agent Instructions (Zeldoc CLI)

The `zeldoc` command-line client for Zeldoc.ai. It talks to `https://api.zeldoc.ai/v1` with
a Zeldoc.ai API key.

## This repository is public
The code and its whole history are public. Reference only public things: the
`api.zeldoc.ai` endpoints and `docs.zeldoc.ai`. Never add internal hostnames or IP
addresses, details of how Zeldoc.ai's infrastructure is built or protected, customer
names, names of private repositories, or secrets, in code, comments, docs, tests or
commit messages. A mistake cannot be undone by a later commit: the history keeps it.

## Endpoints
- **Model metadata comes from `GET /v1/zeldoc/models`**, Zeldoc.ai's model catalog: mode,
  limits, the organization's prices and capabilities for every model the key can call. Don't
  use `/v1/models` for metadata: it only lists names, and aliases such as `zdev` have no
  limits there.
- **Search is `POST /v1/search/zeldoc-search`.**
- **Usage is `GET /v1/zeldoc/usage?period=<today|week|month|last_month>`**: what the key
  itself used. It reports one key only, never the organization's other keys.
- **Key fields are `GET /v1/zeldoc/key`**: the key's own name and its values for the
  organization's key fields (`zeldoc auth fields`). Also one key only, and read-only.
- **Releases come from the release pages on github.com, never GitHub's API**
  (`constants::releases`): the latest release from where `/releases/latest` redirects, the
  installer from `/releases/download/<tag>/`. Without a GitHub login the API allows 60
  requests an hour per IP address, which a shared office network runs out of. axoupdater's
  own update and version check call that API, so `zeldoc update` uses axoupdater only to read
  the install receipt and runs the installer itself.

## Build & Verification Commands
Run inside `flox activate`, in this order: `cargo fmt -> cargo clippy -> cargo test`.
- **Formatting:** `cargo fmt`
- **Linting:** `cargo clippy --all-targets -- -D warnings`
- **Tests:** `cargo test`
- **Typos:** `typos`
- **Try it:** `cargo run -- models`, `cargo run -- usage`, `cargo run -- search <query>`. Set `ZELDOC_CONFIG_DIR` to a
  temporary directory when trying `auth login`/`logout`, so the real saved key is untouched.

## Releasing
Releases are built by [cargo-dist](https://opensource.axo.dev/cargo-dist/) (`dist`, pinned in
the flox environment).
1. Bump `version` in `Cargo.toml`, run `cargo build` so `Cargo.lock` follows, commit.
2. Tag and push: `git tag v0.2.0 && git push origin v0.2.0`.
3. `.github/workflows/release.yml` builds the five targets, the shell and PowerShell
   installers and checksums, and publishes a GitHub Release. The installer URLs in
   README.md always point at the latest release.

`release.yml` is generated: never edit it by hand. Change `dist-workspace.toml` (targets,
installers, install path) and run `dist generate`; CI fails if the workflow is out of date.
To upgrade cargo-dist, bump it in `.flox/env/manifest.toml` and `cargo-dist-version`
together, then run `dist init --yes`.

## Layout
A library (`src/lib.rs`) with a thin binary (`src/main.rs`), feature folders, and **every
struct and enum in its own file**.

```
src/
  main.rs                   parse arguments, run, turn errors into exit status 1
  cli.rs, command.rs        the clap root (`Cli`) and the subcommand enum (`Command`)
  constants.rs              API paths, docs URL, environment variable names
  api_request_handler.rs    authenticated HTTP requests, JSON decoding
  api_request_error.rs      `ApiRequestError`, parsing of API error bodies
  credentials/              key resolution and storage (`CredentialsStore`), `CredentialsError`,
                            `.zeldoc-profile` pins (`ProfilePin`)
    data_transfer_objects/  `ApiKeySecretDTO`, `CredentialsFileDTO`, `ProfileDTO`, `ProfileNameDTO`
  auth/auth_command.rs      `zeldoc auth ...`
  models/                   `zeldoc models`
    models_command.rs       clap arguments, `run`, filtering
    models_service.rs       `ModelsService` trait + `ModelsServiceImpl`
    models_table.rs         table output
    data_transfer_objects/  `ModelDTO`, `ModelIdDTO`, `ModelModeDTO`, ... one per file
  text_table.rs             aligned text tables (`models`, `usage`, `auth fields`)
  terminal_text.rs          `printable`: server text made safe for a terminal
  key_details/              `zeldoc auth fields`
    key_details_command.rs, key_details_service.rs, key_details_text.rs
    data_transfer_objects/  `KeyDetailsDTO`, `KeyFieldDTO`, `KeyFieldValueDTO`, ...
  usage/                    `zeldoc usage`
    usage_command.rs, usage_service.rs, usage_report_text.rs
    data_transfer_objects/  `UsageReportDTO`, `ModelUsageDTO`, `UsageTotalsDTO`, ...
    request_parameters/     `UsagePeriod`
  search/                   `zeldoc search`
    search_command.rs, search_service.rs, search_results_text.rs
    data_transfer_objects/  `SearchResponseDTO`, `SearchResultDTO`
    request_parameters/     `SearchParameters`, `TimeRange`
  update/                   `zeldoc update`: reruns the release installer over the folder in the
                            receipt the installer wrote; no Zeldoc.ai API or GitHub API calls
    update_command.rs, update_service.rs, update_outcome_text.rs, `UpdateError`, `UpdateOutcome`
    release_downloads.rs    the latest release's tag and its installer, from github.com
    installer.rs            runs the installer; on Windows moves the running zeldoc.exe aside
    update_notice.rs        the once-a-day "newer release" line on stderr, terminals only
    data_transfer_objects/  `UpdateCheckDTO`, the cached check; `ReleaseTagDTO`
```

A new feature gets its own folder with `<feature>_command.rs`, `<feature>_service.rs` (trait +
`Impl` taking an `ApiRequestHandler`), `data_transfer_objects/` and, if it sends a body,
`request_parameters/`. `mod.rs` files only declare modules.

## Conventions
- **No abbreviated identifiers:** `response` not `resp`, `api_key` not `key` where it is
  ambiguous.
- **Naming:** API types end in `DTO` (`ModelDTO`), request bodies in `Parameters`
  (`SearchParameters`); file names follow the type (`model_dto.rs`).
- **Strong types everywhere:**
  - Identifiers are newtypes (`ModelIdDTO(pub String)` with `From<String>` and `Display`),
    never a raw `String`.
  - A value from a known set is an enum; when the API may add values, keep an
    `#[serde(untagged)] Unknown(String)` variant (`ModelModeDTO`, `ReasoningEffortDTO`) so a
    new value does not fail the whole response.
  - Money is `rust_decimal::Decimal`, never `f64` or `String`.
  - Strings stay strings only for free text, and for values from many search engines where
    one bad value must not fail the response (search result `url` and `date`).
- **Errors:** services and the credentials store return `thiserror` enums (`ApiRequestError`,
  `CredentialsError`), so callers can match on them (`is_auth_failure`). `anyhow` is only used
  in the `*_command.rs` files and `main.rs`, to add context for the user.
- **Tests** sit in a `#[cfg(test)] mod tests` at the bottom of the file they test; shared
  fixtures in a `#[cfg(test)]` module (`models/test_catalog.rs`).
- **Output:** results go to stdout, messages about the run (`No results.`) to stderr. Every
  listing command has `--json`, which prints a normalized array, not the raw API response.
  Rendering functions take `&mut impl Write` so they can be tested without a network.
- **Exit status:** 0 on success, 1 on any error. `auth status` also exits 1 when there is no
  usable key, so scripts and agents can check it.
- **Key precedence** (`credentials_store.rs`): `--profile`/`ZELDOC_PROFILE`, then a
  `.zeldoc-profile` pin, then `ZELDOC_API_KEY`, then the default profile. An unsaved
  requested or pinned profile is an error, never a fall-through to another key: keys
  belong to different customers. Change the order only together with the README and
  `opencode-zeldoc`, which reads pins the same way.
- **Text other people typed in is untrusted.** Key names, key field names and values are
  set by an organization's admins: pass them through `terminal_text::printable` before
  printing them as text, so an escape sequence or line break in one cannot control the
  terminal. `--json` escapes them already.
- **The API key never appears in arguments, logs or errors.** Read it from the environment, the
  credentials file, a hidden prompt or stdin, and keep it in an `ApiKeySecretDTO`, whose
  `Debug` is redacted. Show it only through `masked()`; `expose()` is for sending it and for
  `auth token`, whose job is to print it.
- **clap help text lives on the `Command` variants**, not on the `*Command` argument structs:
  a doc comment on the struct would compete with the variant's. Use `//` comments there.
- **Help text is documentation for agents.** Agents learn the CLI from `--help`, so doc comments
  on commands and flags must be accurate and say what a flag really does (e.g. which search
  filters are not guaranteed).
