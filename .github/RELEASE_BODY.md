## Downloads

### Desktop app
A small window that lets you pick a markdown file and rebuilds the PDF every time you save it.

| File | Platform |
|---|---|
| **`The Sieve_<version>_aarch64.dmg`** | macOS — Apple Silicon (M1 / M2 / M3 / M4) |
| **`The Sieve_<version>_x64.dmg`** | macOS — Intel |
| **`the-sieve-app_<version>_amd64.AppImage`** | Linux — universal (run `chmod +x` then double-click) |
| **`the-sieve-app_<version>_amd64.deb`** | Debian / Ubuntu |
| **`The Sieve_<version>_x64-setup.exe`** | Windows — recommended installer (NSIS) |
| **`The Sieve_<version>_x64_en-US.msi`** | Windows — MSI installer (better for Group Policy / silent installs) |

### Command-line binary
For terminal users and scripting. `the-sieve <input.md>` converts markdown into a half-letter PDF.

| File | Platform |
|---|---|
| **`the-sieve-aarch64-apple-darwin`** | macOS — Apple Silicon |
| **`the-sieve-x86_64-apple-darwin`** | macOS — Intel |
| **`the-sieve-x86_64-unknown-linux-gnu`** | Linux |
| **`the-sieve-x86_64-pc-windows-msvc.exe`** | Windows |

Each binary ships with a `.sha256` companion file for integrity verification. On Unix systems make the binary executable with `chmod +x` before running.

### macOS note
The desktop app is ad-hoc signed but **not** notarized (notarization requires a paid Apple Developer account). The first time you open it, macOS may say it's from an unidentified developer — right-click the app and choose **Open** to bypass, or run:

```sh
xattr -cr "/Applications/The Sieve.app"
```

The CLI binary has no signature and may need the same `xattr` treatment.

---
