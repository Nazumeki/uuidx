# Installation

This guide is for users who want to install the published `uuidx` CLI. It
covers two installation paths: direct manual installation from a release
archive, and manual installation through Cargo from crates.io. It also includes
non-interactive variants for AI agents. To compile from source or choose custom
features and targets, use
[`SOURCEBUILD.md`](SOURCEBUILD.md).

The installed executable is named `uuidx` on Unix-like systems and
`uuidx.exe` on Windows.

## Choose an installation method

| Method          | Use when                                                                                       |
| --------------- | ---------------------------------------------------------------------------------------------- |
| Release archive | Rust is unavailable, or you want the exact binary published for your platform.                 |
| crates.io       | Rust and Cargo are already installed and you want Cargo to manage the binary.                  |
| AI agent recipe | An automated environment needs a deterministic, non-interactive install and verification step. |

## Direct manual installation

Use a release archive when Rust and Cargo are unavailable, or when you want to
install the exact binary published for your platform. The steps below verify the
archive, place the executable in a user-owned directory, and configure `PATH`.

Release archives are published for these targets:

| Platform            | Target                     | Archive   |
| ------------------- | -------------------------- | --------- |
| Linux x86_64        | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Windows x86_64      | `x86_64-pc-windows-msvc`   | `.zip`    |
| macOS Intel         | `x86_64-apple-darwin`      | `.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin`     | `.tar.gz` |

Choose the archive matching both the operating system and CPU architecture.
Other targets are not published as release archives; use a source build from
[`SOURCEBUILD.md`](SOURCEBUILD.md) for those targets.

### Unix-like systems

Set `VERSION` to the release tag you want, such as `v0.1.1`, and set `TARGET`
to the target for your machine:

```console
VERSION=v0.1.1
TARGET=x86_64-unknown-linux-gnu
ARCHIVE="uuidx-${VERSION}-${TARGET}.tar.gz"
BASE_URL="https://github.com/Nazumeki/uuidx/releases/download/${VERSION}"

curl --fail --location --remote-name "${BASE_URL}/${ARCHIVE}"
curl --fail --location --remote-name "${BASE_URL}/SHA256SUMS"
grep "  ${ARCHIVE}$" SHA256SUMS | sha256sum --check -

tar -xzf "${ARCHIVE}"
mkdir -p "$HOME/.local/bin"
cp uuidx "$HOME/.local/bin/uuidx"
chmod 755 "$HOME/.local/bin/uuidx"
export PATH="$HOME/.local/bin:$PATH"

uuidx --version
```

The `export` applies to the current shell. To make the setting persistent, add
the following line to the startup file used by your shell, usually
`~/.profile`, `~/.bashrc`, or `~/.zshrc`:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Do not skip checksum verification when the archive is used in automation. The
release workflow publishes `SHA256SUMS` next to every archive.

### Windows PowerShell

Set `VERSION` to the release tag you want. The archive contains `uuidx.exe`.

```powershell
$Version = "v0.1.1"
$Target = "x86_64-pc-windows-msvc"
$Archive = "uuidx-$Version-$Target.zip"
$BaseUrl = "https://github.com/Nazumeki/uuidx/releases/download/$Version"

Invoke-WebRequest "$BaseUrl/$Archive" -OutFile $Archive
Invoke-WebRequest "$BaseUrl/SHA256SUMS" -OutFile SHA256SUMS

$Expected = (Select-String -Path SHA256SUMS -Pattern ([regex]::Escape($Archive))).Line.Split()[0]
$Actual = (Get-FileHash $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ($Actual -ne $Expected) { throw "SHA-256 mismatch for $Archive" }

$InstallDir = Join-Path $env:USERPROFILE ".local\bin"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Expand-Archive -Path $Archive -DestinationPath $InstallDir -Force

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($UserPath -split ";") -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
}
$env:Path = "$InstallDir;$env:Path"

uuidx.exe --version
```

Open a new terminal after changing the user `PATH`. The last assignment only
updates the current PowerShell process.

## Install with Cargo

Cargo provides a second manual installation path. It downloads the published
`uuidx-cli` package, builds it locally, and places the `uuidx` executable in
Cargo's binary directory:

```console
cargo install uuidx-cli --locked
uuidx --version
```

To install a specific published version:

```console
cargo install uuidx-cli --version 0.1.1 --locked
```

The CLI enables the `ulid-inspect` feature by default. Install a smaller binary
without that optional feature with:

```console
cargo install uuidx-cli --locked --no-default-features
```

If `uuidx` is not found after installation, add Cargo's binary directory to
`PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

On Windows, add `%USERPROFILE%\.cargo\bin` to the user `PATH` through System
Environment Variables or PowerShell:

```powershell
$CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($UserPath -split ";") -notcontains $CargoBin) {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$CargoBin", "User")
}
$env:Path = "$CargoBin;$env:Path"
```

## Installation for AI agents

Use a fixed install root and explicit JSON output when an AI agent will invoke
`uuidx`. This avoids relying on a shell profile or terminal detection.

```console
cargo install uuidx-cli --locked --root "$HOME/.local"
export PATH="$HOME/.local/bin:$PATH"
uuidx --version
uuidx generate v4 --output json
```

For a feature-minimal agent install, add `--no-default-features` to the
`cargo install` command. For a platform without Cargo, use the direct manual
installation path above and select the matching release target.

When invoking the CLI from automation:

- Use `--output json` for structured results and read one JSON object per line.
- Provide values as arguments, files, or piped newline-delimited input; do not
  depend on interactive stdin.
- Treat exit status `1` as one or more invalid input records, `2` as a usage or
  missing-input error, and `3` as an I/O or serialization error.
- Run `uuidx --version` after installation and before using the binary.

The complete JSONL contract is in [`JSON.md`](JSON.md).
