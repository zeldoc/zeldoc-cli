# zeldoc

Command-line client for [Zeldoc.ai](https://zeldoc.ai), the EU-sovereign LLM API.

It is small on purpose. It exposes the parts of Zeldoc.ai that are useful from a shell,
and coding agents such as Claude Code, Codex, OpenCode or Pi can use it with no plugin:
any agent that can run a command can list models, check its usage or search the web
through Zeldoc.ai.

## Install

**macOS and Linux:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/zeldoc/zeldoc-cli/releases/latest/download/zeldoc-installer.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/zeldoc/zeldoc-cli/releases/latest/download/zeldoc-installer.ps1 | iex"
```

The installers download the prebuilt binary for your platform from the
[latest release](https://github.com/zeldoc/zeldoc-cli/releases/latest), check its
checksum and put it in `~/.local/bin` (`%USERPROFILE%\.local\bin` on Windows), or in
`$XDG_BIN_HOME` when that is set, adding the directory to your `PATH` if needed. Open a
new terminal afterwards. No Rust toolchain is needed.

Releases have binaries for macOS (Apple silicon and Intel), Linux (x86_64 and ARM64) and
Windows (x86_64); you can also download an archive from the release page yourself. To
build from source instead: `cargo install --path .`

## Update

```bash
zeldoc update            # install the latest release in place of this one
zeldoc update --check    # only say whether a newer release exists
```

`zeldoc update` downloads the latest release's installer and runs it, so the new version
replaces the old one in the same folder. It works for copies installed with the commands
above. A copy built with `cargo install` or unpacked from an archive by hand is updated
the way it was installed. Both the update and the check below use the release pages on
github.com, not GitHub's API, so they work on networks where many people share one IP
address and GitHub's limit for API requests without a login has run out.

Once a day, a command run in a terminal also checks GitHub for a newer release, in the
background, and prints a line to stderr when there is one. Commands whose output goes to a
pipe or a file, as in scripts and coding agents, never check. Set `ZELDOC_NO_UPDATE_CHECK=1`
to turn the check off.

## Authenticate

Save an API key once. It is checked against Zeldoc.ai before it is stored:

```bash
zeldoc auth login
```

The key is saved as the profile `default`; see [One key per customer](#one-key-per-customer)
for more than one key. The key is read from a hidden prompt, or from standard input when input is piped
(`zeldoc auth login < key.txt`). Don't have a key yet? See
[Generate an API key](https://docs.zeldoc.ai/connect-opencode#generate-an-api-key).

`ZELDOC_API_KEY`, when set, takes precedence over the default saved key, so existing
setups keep working without a login.

| Command | Does |
|---|---|
| `zeldoc auth login` | Checks and saves a key, as profile `default` or the one given with `--profile` |
| `zeldoc auth list` | Lists the saved profiles, marking the default and the one pinned for this folder |
| `zeldoc auth use <profile>` | Makes a saved profile the default |
| `zeldoc auth pin <profile>` | Pins the current folder to a profile (writes `.zeldoc-profile`) |
| `zeldoc auth status` | Shows which key is in use (masked), why, and whether it works; exits 1 if not |
| `zeldoc auth fields` | Shows the key fields your organization set on the key, such as its team or project |
| `zeldoc auth token` | Prints the key, for tools that read `ZELDOC_API_KEY` |
| `zeldoc auth logout` | Deletes the saved key of the profile in use |

### One key per customer

If you work for several customers, each with their own Zeldoc.ai key, save each key under
a profile name and pin each customer's repository to its profile:

```bash
zeldoc auth login --profile acme      # save Acme's key
zeldoc auth login --profile globex    # save Globex's key

cd ~/work/acme-app
zeldoc auth pin acme                  # writes .zeldoc-profile containing "acme"
zeldoc usage                          # uses Acme's key here and in every folder below
```

`.zeldoc-profile` holds only the profile name, never a key, so you can commit it. Everyone
on the team then saves their own key under that name by running `zeldoc auth login` in
the repository; with a pin and no `--profile`, `login` saves under the pinned name.

The CLI picks the key in this order:

1. `--profile <name>` (or `-P`), or the `ZELDOC_PROFILE` variable
2. `.zeldoc-profile` in the current folder or the nearest folder above it
3. `ZELDOC_API_KEY`
4. the default profile: the first key you saved, or the one set with `zeldoc auth use`

A pin wins over `ZELDOC_API_KEY`, so it still applies when your shell profile exports the
variable. If a pin names a profile that is not saved, the CLI stops with an error instead
of using another customer's key. On a machine with no saved keys, such as CI, pins are
ignored and `ZELDOC_API_KEY` is used. `zeldoc auth status` shows which key is used and
why.

To hand the pinned key to other tools started in the repository, such as OpenCode or an
OpenAI SDK, use [direnv](https://direnv.net) (Linux and macOS) with this `.envrc`:

```bash
export ZELDOC_API_KEY="$(zeldoc auth token)"
```

`zeldoc usage --all` reports the usage of every saved profile, one row each, with a total.

### Where the key is stored

All saved keys are in `credentials.json` in the platform config directory:

| Platform | Path |
|---|---|
| Linux | `~/.config/zeldoc/credentials.json` |
| macOS | `~/Library/Application Support/zeldoc/credentials.json` |
| Windows | `%APPDATA%\zeldoc\credentials.json` |

`ZELDOC_CONFIG_DIR` overrides the directory. On Unix the file is created with mode `0600`
in a `0700` directory, so only your user can read it. It is not encrypted: like
`~/.codex/auth.json` or `~/.aws/credentials`, anything running as your user, including an
AI agent with shell access, can read it. The same is true of `ZELDOC_API_KEY` in your
environment.

### Using the saved key in other tools

`zeldoc auth token` prints the key the CLI uses: `ZELDOC_API_KEY` if it is set, otherwise
the key saved by `zeldoc auth login`. It does not create a new key and makes no network
call, so it is cheap to run every time a shell starts.

A program cannot set environment variables in the shell that started it, so
`zeldoc auth login` cannot set `ZELDOC_API_KEY` for you. Set it from `zeldoc auth token`
in your shell profile instead, so every new shell has it. To set it in the current shell
only, run the same line there.

**Linux and macOS (bash, zsh).** Add this line to `~/.bashrc` or `~/.zshrc` (zsh is the
default shell on macOS):

```bash
export ZELDOC_API_KEY="$(zeldoc auth token)"
```

**fish.** Add this line to `~/.config/fish/config.fish`:

```fish
set -gx ZELDOC_API_KEY (zeldoc auth token)
```

**Windows (PowerShell).** Add the line to your PowerShell profile, creating the profile
first if you don't have one:

```powershell
if (-not (Test-Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE -Force | Out-Null }
Add-Content -Path $PROFILE -Value '$env:ZELDOC_API_KEY = zeldoc auth token'
```

If new PowerShell windows then report that running scripts is disabled, the execution
policy is blocking your profile. `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`
allows local scripts such as the profile to run.

**Programs started from the Start menu or the Dock** don't read shell profiles. On Windows,
set a user environment variable instead:

```powershell
[Environment]::SetEnvironmentVariable("ZELDOC_API_KEY", (zeldoc auth token), "User")
```

This stores a copy of the key in your Windows user environment. Run it again after you
change key, then restart the programs that use it.

Setting the variable from `zeldoc auth token` keeps the key out of your profile file,
which often ends up in a dotfiles repository. The key is still in the environment of every
program started from that shell, as with a hand-written `export`.

After `zeldoc auth login` with a new key, open a new shell: shells that are already open
keep the old value, and `ZELDOC_API_KEY` takes precedence over the saved key.

## Show a key's fields

```bash
zeldoc auth fields
zeldoc auth fields --all
zeldoc auth fields --json
```

Organizations can label their API keys with fields, such as a team, a project or whether
the key is private; their admins set them in the dashboard. `zeldoc auth fields` shows the
key's name and each of the organization's fields with this key's value, from Zeldoc.ai's
key endpoint (`GET /v1/zeldoc/key`). Only the key the CLI uses is shown; `--all` shows the
key of every saved profile. `--json` prints each value as the API sends it: a select
field's option key, with its label in `value_label`.

## List models

```bash
zeldoc models
zeldoc models --mode chat
zeldoc models --json
```

Lists the models your key can use from Zeldoc.ai's model catalog
(`GET /v1/zeldoc/models`): kind, context window, output limit, your organization's price
per 1 million tokens, and capabilities. `--json` adds cache prices and the accepted
`reasoning_effort` values. `--mode` filters on the kind: `chat`, `responses`,
`embedding`, `image_generation`, `audio_transcription` or `realtime`.

The gateway's own `GET /v1/models` only lists names; it has no limits or prices for
aliases such as `zdev`, which is why the CLI uses the catalog.

## Show usage

```bash
zeldoc usage
zeldoc usage --period today
zeldoc usage --period last-month --json
```

Shows what your API key has used from Zeldoc.ai's usage endpoint
(`GET /v1/zeldoc/usage`): requests, input, output and cached tokens, and cost per model,
with a total. Costs are in USD, as the dashboard shows them; Zeldoc.ai's own models are
covered by the subscription and cost 0. `--period` is `today`, `week`
(the last 7 days), `month` (this calendar month, the default) or `last-month`, all in UTC.
`--profile` picks another saved key, and `--all` reports every saved profile instead.
When the key has a monthly spend limit, the report shows how much of it is used, and for
organizations on prepaid credits, the credits left. For a key that belongs to a ZDev seat,
it shows how much of the plan's monthly tokens are used. `--json` prints exact costs and cache
writes.

Only the key the CLI uses is reported, never other keys of your organization; the
dashboard at [app.zeldoc.ai](https://app.zeldoc.ai) shows the whole organization. New
requests can take a minute to appear.

## Search the web

```bash
zeldoc search podman rootless ports below 1024
zeldoc search --engines github,stackoverflow -n 5 axum middleware
zeldoc search --json --time-range week rust 2024 edition
```

Calls Zeldoc.ai's search endpoint (`POST /v1/search/zeldoc-search`). General searches use
Staan, an EU-based provider; `--categories it` and `--engines` can reach services outside
the EU. See [Web search](https://docs.zeldoc.ai/web-search) for what leaves your machine,
and keep personal or confidential information out of queries.

## Configuration

| Variable | Default | Purpose |
|---|---|---|
| `ZELDOC_PROFILE` | | Saved profile to use, like `--profile` |
| `ZELDOC_API_KEY` | | API key; used where no profile is picked by `--profile`, `ZELDOC_PROFILE` or a pin |
| `ZELDOC_CONFIG_DIR` | platform config directory + `/zeldoc` | Where the saved key lives |
| `ZELDOC_NO_UPDATE_CHECK` | | Any value turns off the daily check for a newer release |

## License

[MIT](LICENSE)
