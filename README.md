# Grits

[![Build status](https://github.com/solidiquis/grits/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/solidiquis/grits/actions)
[![Crates.io](https://img.shields.io/crates/v/grits.svg)](https://crates.io/crates/grits)
[![Crates.io](https://img.shields.io/crates/d/grits)](https://crates.io/crates/grits)

A simple line-text formatter that makes it simple to parse, filter, and format live logs, turning noise into meaningful insights.

![demo gif](images/log.gif)

<p align="center">
  <em>An example of Grits in action: the left pane shows Grits being applied, and the right pane displays the raw logs.</em>
</p>


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


## Documentation

The documentation for `grits` can be found [here](./docs/help.md). It is also available in the CLI via `--help`.

## Contributing

All well-intentioned forms of contributions are welcome.

## Donating

If you like this tool, please consider [buying me a coffee](https://buymeacoffee.com/O3nsHqb7A9). Much appreciated!

## FAQ

Q: **Why is this called grits?**
A: I was really craving shrimp & grits while writing this on a plane going to South Korea.
