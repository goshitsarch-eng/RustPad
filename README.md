# RustPad

RustPad is a small text editor built with Rust and Iced. It keeps the old-school Notepad feel, but adds a few modern conveniences so it stays useful for everyday editing.

## Features

RustPad currently supports:

- Open, save, and save as for UTF-8 text files
- Find, replace, and go to line
- Font selection, font style selection, and font size changes
- Word wrap and dark mode toggles
- A right-click context menu for common edit actions
- Time/date insertion with `F5`
- Print support through the system `lp` command
- A single-level undo snapshot, similar to classic Notepad behavior
- A `.LOG` workflow that appends a timestamp when opening files that start with `.LOG`

## Install

If you use Arch, RustPad is on the AUR:

`paru -S rustpad`

or

`yay -S rustpad`

If you just want a ready made build, grab one from GitHub Releases:

`https://github.com/goshitsarch-eng/RustPad/releases`

Current release artifacts are:

- Linux AppImage for x86_64
- Linux AppImage for aarch64
- Windows NSIS installer for x86_64
- macOS `.app` bundle for aarch64
- macOS `.dmg` package for aarch64

The release workflow publishes these assets for tagged releases that start with `v`.

## Build It Yourself

To run RustPad from source:

`cargo run`

To build an optimized release binary:

`cargo build --release`

The final binary ends up at:

`target/release/rustpad`

If you want to package it locally, the repo is configured for `cargo packager` outputs in `dist/` using the `appimage`, `nsis`, `app`, and `dmg` formats.

Printing goes through your system `lp` setup, so that part only works if your machine already has a printer configured and the print service is running.
