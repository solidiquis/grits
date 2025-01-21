# Grits

[![Build status](https://github.com/solidiquis/grits/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/solidiquis/grits/actions)
[![Crates.io](https://img.shields.io/crates/v/grits.svg)](https://crates.io/crates/grits)
[![Crates.io](https://img.shields.io/crates/d/grits)](https://crates.io/crates/grits)

A simple line-text formatter that makes it simple to parse, filter, and format live logs, turning noise into meaningful insights.

![demo gif](images/log.gif)

<p align="center">
  <em>An example of Grits in action: the left pane shows Grits being applied, and the right pane displays the raw logs.</em>
</p>


* [Installation](#installation)
* [Documentation](#documentation)
* [Contributing](#contributing)
* [Donating](#donating)
* [FAQ](#faq)

At its core, `grits` applies regular expressions with named captures to input lines. These captures are then available as variables
(a.k.a. anchors) which can then be used in a `grits` template string. The template string supports text-alignment, colorization,
as well as other attributes you'd expect using ANSI escape sequences.

The following example demonstrates how to apply `grits` to `tcpdump` to extract an output line's timestamp (`ts`) and
a packet's source (`src`) and destination (`dst`) IP address:

```bash
tcpdump -nn | grits -- \
  -p '^(?<ts>[^ ]+)' \
  -p 'IP\w? (?<src>[^ ]+)' \
  -p '> (?<dst>[^ ]+):' \
  -t '[{(cyan|bold):ts}] {(green|underlined):"src"}={(lalign(45)):src} {(yellow|underlined):"dst"}={dst}'
```

![demo image](images/demo.png)
<p align="center">
  <em>The top pane in the above screenshot is the raw output of tcpdump while the bottom pane shows the output being piped into grits.</em>
</p>

## Installation

### crates.io

```bash
cargo install grits
```

### cURL

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/solidiquis/grits/releases/download/v0.3.0/grits-installer.sh | sh
```

### Powershell

```bash
powershell -ExecutionPolicy Bypass -c "irm https://github.com/solidiquis/grits/releases/download/v0.3.0/grits-installer.ps1 | iex"
```

### Manual installation

Check the [releases page](https://github.com/solidiquis/grits/releases) for prebuilt binaries.


## Documentation

The documentation for `grits` can be found [here](./docs/help.md). It is also available in the CLI via `--help`.

## Contributing

All well-intentioned forms of contributions are welcome.

## Donating

If you like this tool, please consider [buying me a coffee](https://buymeacoffee.com/O3nsHqb7A9). Much appreciated!

## FAQ

- Q: **Have you heard of lnav? Why should I use grits over lnav?**
- A: I have heard of [lnav](https://github.com/tstack/lnav) but I haven't used it. Users are encouraged to use both to see what best fits their needs.

- Q: **Why is this called grits?**
- A: I was really craving shrimp & grits while writing this on a plane going to South Korea. Also, checkout my Spotify playlist called [Melancholy with a side of Grits](https://open.spotify.com/playlist/2bsBVlDXS1yWwUjxuSRtd2?si=44122d2dc11b4a90).

