# Grits

[![Build status](https://github.com/solidiquis/grits/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/solidiquis/grits/actions)
[![Crates.io](https://img.shields.io/crates/v/grits.svg)](https://crates.io/crates/grits)
[![Crates.io](https://img.shields.io/crates/d/grits)](https://crates.io/crates/grits)

`grits` is a minimal, simple, and easy-to-use line text-formatter that applies regular expressions with named captures to input lines
and transforms them using a template string. It allows for efficient parsing, extracting, and formatting of text,
including support for colorization and other attributes you'd get using ANSI escape sequences.

The following example demonstrates how to apply `grits` to `tcpdump` to extract a packet's source (`src`) and destination (`dst`) IP address:

```bash
sudo tcpdump -nn | grits -p '^(?<ts>[^ ]+)' \
  -p 'IP\w? (?<src>[^ ]+)' \
  -p '> (?<dst>[^ ]+):' \
  -t '[{(cyan|bold):ts}] {(green|underlined):"src"}={src} {(yellow|underlined):"dst"}={dst}'
```

![demo image](images/demo.png)
The top pane in the above screenshot is the raw output of `tcpdump` while the bottom pane shows the output being piped into `grits`.
