```markdown
# Cosmic Nightlight

A native Rust applet for the COSMIC desktop environment that provides a graphical slider interface to control screen temperature and reduce blue light.

## Prerequisites

This applet requires **gammastep** to be installed on your host system to process and apply the color temperature adjustments:

```sh
sudo apt install gammastep

```

## Known Technical Constraints (Alpha Roadmap)

This project utilizes an Elm-style Model-View-Update (MVU) design pattern to bridge a graphical slider frontend with system utilities.

Because this application targets the modern **COSMIC Desktop Environment**, its background color-shifting capabilities rely on the compositor (`cosmic-comp`) implementing standard Wayland gamma protocols (`wlr-gamma-control-unstable-v1`). During the current COSMIC development phases, external gamma controls may be restricted by the display server's hardware abstractions. The applet frontend is structured natively to seamlessly apply smooth color transitions as the compositor protocols mature.


## Installation

A [justfile](https://www.google.com/search?q=./justfile) is included by default for the [casey/just](https://github.com/casey/just) command runner.

* `just` builds the application with the default `just build-release` recipe
* `just run` builds and runs the application
* `just install` installs the project into the system
* `just vendor` creates a vendored tarball
* `just build-vendored` compiles with vendored dependencies from that tarball
* `just check` runs clippy on the project to check for linter warnings
* `just check-json` can be used by IDEs that support LSP

## Translators

[Fluent](https://projectfluent.org/) is used for localization of the software. Fluent's translation files are found in the [i18n directory](https://www.google.com/search?q=./i18n). New translations may copy the [English (en) localization](https://www.google.com/search?q=./i18n/en) of the project, rename `en` to the desired [ISO 639-1 language code](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes), and then translations can be provided for each [message identifier](https://projectfluent.org/fluent/guide/hello.html). If no translation is necessary, the message may be omitted.

## Packaging

If packaging for a Linux distribution, vendor dependencies locally with the `vendor` rule, and build with the vendored sources using the `build-vendored` rule. When installing files, use the `rootdir` and `prefix` variables to change installation paths.

```sh
just vendor
just build-vendored
just rootdir=debian/cosmic-nightlight prefix=/usr install

```

It is recommended to build a source tarball with the vendored dependencies, which can typically be done by running `just vendor` on the host system before it enters the build environment.

## Developers

Developers should install [rustup](https://rustup.rs/) and configure their editor to use [rust-analyzer](https://rust-analyzer.github.io/).