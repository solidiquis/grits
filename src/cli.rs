use anyhow::{format_err, Result};
use clap::{crate_authors, crate_version, Parser};
use clap_complete::Shell;
use std::{env, str::FromStr};

#[derive(Parser, Debug)]
#[command(
    author = crate_authors!(),
    version = crate_version!(),
    about = "
A text line processor that applies regular expressions with named captures to input lines
and transforms them using a user-generated template. See the long help '--help' for further
details and examples.",
    long_about = "
Sections:
    - INTRODUCTION
    - COLORIZATION
    - OUTPUT TEMPLATE
    - ATTRIBUTES
    - EXAMPLES

------------------
---INTRODUCTION---
------------------
Grits is a line processor that applies regular expression containing named captures to
input lines and transforms them using a custom template string. For example, say we had
the following input line which is stored in `$LINE`:

    kernel=Memory: 125312k/129720k available (1540k kernel code sysctl=NONE

All we're really interested in is how much memory is left, the rest can be considered noise.
We'll apply the following regular expression against the input line to extract the remaining
memory:

    Memory: (?<mem>[^ ]+)

The remaining memory is stored in a capture called 'mem'. Now to transform the input line into
our desired output, we'll use the following template string:

    Remaining memory: ${mem}

Putting everything together the following command:

    $ echo $LINE | grits -p 'Memory: (?<mem>[^ ]+)' -o 'Remaining memory: ${mem}'

Gets us the following output:

    Remaining memory: 125312k/129720k

------------------------
------COLORIZATION------
------------------------

Colorization is disabled under two circumstances:

    1. If the `NO_COLOR` environment variable is set and not blank.
    2. Standard output is not a terminal.

An example of how to disable color:

    $ NO_COLOR=1 grits -p 'Memory: (?<mem>[^ ]+)' -o 'Remaining memory: ${mem}' file

------------------------
----OUTPUT TEMPLATE-----
------------------------
The output template which is specified using `-o, --output-template` is a format string
that defines how to transform a line input and ultimately produce an output. The output
template contains constructs called anchors which looks like the following:
    
    ${log}

This anchor, which begins with '$' and ends with '}', is a reference to a named capture
from a regular expression that a user provides. If, for a given input line, there is a
match for the `log` capture, then the value associated with the `log` capture will be used
to interpolate the output string at the `log` anchor. For example:

    $ echo 'level=info msg=foobar path=/baz' | grit -p 'msg=(?<log>[^ ]+)' -o 'transformed=${log}'

Will get you the following output:

    transformed=foobar

Anchors come with a handful of other features as well:

1. Indexing in the case of multiple matches for a given a single named capture group:

    ${log[1]}

2. Chain default values, which can be another anchor or a string literal. The first non-blank
value will be used in the output (ultimately falls back to an empty string):

    ${log || foo || bar[1] || \"NO MATCH\"}

3. Apply to an anchor like so:

    ${(red|bold):foo}

4. Apply ANSI-escape sequences to literals:

    ${(cyan|underlined):\"foo\"}

5. Escape characters with special meaning such as '$' with '\\':

    USD: \\$${amount}

----------------
---ATTRIBUTES---
----------------
As mentioned in the section prior, attributes are available to stylize the result of processing
the anchors. Multiple attributes may be used together like so:

    ${(red|bold|underlined):foo}

The following attributes are currently available:

- black (apply a black foreground)
- red (apply a red foreground)
- green (apply a green foreground)
- yellow (apply a yellow foreground)
- blue (apply a blue foreground)
- magenta (apply a magenta foreground)
- cyan (apply a cyan foreground)
- white (apply a white foreground)
- bg_black (apply a black background)
- bg_red (apply a red background)
- bg_green (apply a green background)
- bg_yellow (apply a yellow background)
- bg_blue (apply a blue background)
- bg_magenta (apply a magenta background)
- bg_cyan (apply a cyan background)
- bg_white (apply a white background)
- bold (make text bold)
- underlined (underline text)
- reverse (reverse text)
- crossed_out (crossout text)

----------------
----EXAMPLES----
----------------
1. Multi-file processing:

    $ grits -p 'sysctl=(?<sysctl>.*)'` -p 'sysctl output: ${sysctl}' file1 file2

2. Piping:

    $ docker logs -f 93670ea0964c | grits -p 'log_level=info(?<log>.*)' -o 'INFO LOG: ${log}'

3. Attributes, default values, and multiple regular expressions:

    $ kubectl logs -f -n foo -l app=bar | grits \\
         -p '^kernel:(?<kern>.*)' \\
         -p '^sysctl:(?<sys>.*)' \\
         -o kernel=${(cyan):kern || \"NONE\"} sysctl=${(magenta):sys || \"NONE\"}
"
)]
pub struct Cli {
    /// A regular expression with named captures. Can be specified multiple times
    #[arg(short, long)]
    pub pattern: Vec<String>,

    /// A template string that defines how to transform a line input. See long '--help'
    #[arg(short, long)]
    pub template: String,

    /// Input files
    pub files: Vec<String>,

    /// Name of capture that must have at least one match for the output to show. Can be specified
    /// multiple times
    #[arg(short, long)]
    pub require: Vec<String>,

    /// Force output to be line-buffered. By default, output is line buffered when stdout is a
    /// terminal and block-buffered otherwise
    #[arg(long)]
    pub line_buffered: bool,

    /// Produce completions for shell and exit
    #[arg(short, long)]
    pub completions: Option<clap_complete::Shell>,
}

impl Cli {
    pub fn compute_shell_used_for_completions() -> Result<Option<Shell>> {
        let mut raw_args = env::args_os();

        if raw_args.any(|a| {
            a.as_os_str()
                .to_str()
                .is_some_and(|s| s == "--completions" || s == "-c")
        }) {
            if let Some(raw_shell) = raw_args.next().and_then(|a| a.as_os_str().to_str().map(String::from)) {
                let shell = Shell::from_str(&raw_shell.to_lowercase())
                    .map_err(|e| format_err!("failed to determine which Shell to generate autocomplete due to {e}"))?;
                return Ok(Some(shell));
            }
        }
        Ok(None)
    }
}
