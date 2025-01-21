# Grits Documentation

* [Usage](#usage)
* [Templating language](#templating-language)
    - [Indexing](#indexing)
    - [Default values](#default-values)
    - [Attributes](#attributes)
    - [Conditional attributes](#conditional-attributes)
    - [Filtering](#filtering)
    - [Other examples](#other-examples)
* [Completions](#completions)
* [Colorization](#colorization)

## Usage

```
A text line processor that applies regular expressions with named captures to input lines
and transforms them using a user-generated template. See the long help '--help' for further
details and examples.

Usage: grits [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Input files

Options:
  -p, --pattern <PATTERN>            A regular expression with named captures. Can be specified multiple times
  -t, --template <TEMPLATE>          A template string that defines how to transform a line input using times. Can be specified multiple times. See long '--help'
  -s, --separator <SEPARATOR>        Separator used to join results of transforming each template if multiple are specified [default: ]
  -r, --require <REQUIRE>            Comma-separated capture names that must have a match for a given input line to be processed; otherwise it is ignored
      --require-mode <REQUIRE_MODE>  Modify '-r, --require' to require matching on all specified capture names or any [default: all] [possible values: all, any]
      --line-buffered                Force output to be line-buffered. By default, output is line buffered when stdout is a terminal and block-buffered otherwise
  -c, --completions <COMPLETIONS>    Produce completions for shell and exit [possible values: bash, elvish, fish, powershell, zsh]
  -h, --help                         Print help (see more with '--help')
  -V, --version                      Print version
```

## Templating language

`grits` uses a simple templating language to transform text. Templates consist of anchors.
Anchors are placeholders enclosed within `{...}` that correspond to named capture groups from
the regular expression applied to the input. Once a match is found, the value from the
capture group is inserted into the anchor’s position in the template string.

Here's an example:
```bash
echo 'level=info msg=foobar path=/baz' | grit -p 'msg=(?<log>[^ ]+)' -t 'transformed={log}'
```

In this command, we use a regular expression to capture the value associated with the msg field.
The capture group is named `log`. The template string `transformed={log}` will replace `{log}` with
the value captured from the input. The output will then be:

```
transformed=foobar
```

To summarize:
- The regular expression `msg=(?<log>[^ ]+)` captures the value `foobar` into the `log` capture group.
- The template `transformed={log}` uses the value of `log` to generate the output.

The following are additional features of `grits` templating system:

### Indexing

When there are multiple matches for a given named capture group, you can use **indexing**
to specify which occurrence of the capture to use. The index is placed in square brackets
immediately after the anchor name.  For example, to access the second match of the `log`
capture group, you would use:

```
{log[1]}
```

### Default values

If a particular anchor doesn't have an associated match, default values can be chained using the `||`
operator like so:

```
{log || foo || bar[1] || "NO MATCH"}
```

The first default value that doesn't produce a blank string will be used. Default values can be
other anchors or a string literal.

### Attributes

Attributes offer additional means to transform text. Attributes are applied to anchors like so:

```
{(red|bold):ipaddr_v4}
```

Here is an example using attributes with default values:


```
{(red|bold):ipaddr_v4 || ipaddr_v6 || "NOMATCH"}
```

In the above example, `red` and `bold` will be applied the entire anchor.


The following attributes are currently available:

- `black` (apply a black foreground)
- `red` (apply a red foreground)
- `green` (apply a green foreground)
- `yellow` (apply a yellow foreground)
- `blue` (apply a blue foreground)
- `magenta` (apply a magenta foreground)
- `cyan` (apply a cyan foreground)
- `white` (apply a white foreground)
- `bg_black` (apply a black background)
- `bg_red` (apply a red background)
- `bg_green` (apply a green background)
- `bg_yellow` (apply a yellow background)
- `bg_blue` (apply a blue background)
- `bg_magenta` (apply a magenta background)
- `bg_cyan` (apply a cyan background)
- `bg_white` (apply a white background)
- `bold` (make text bold)
- `underlined` (underline text)
- `reverse` (reverse text)
- `crossed_out` (crossout text)
- `lalign(number)` (left aligns text using specified argument as the width and an empty space, `' '` as the fill character)
- `ralign(number)` (right aligns text using specified argument as the width and an empty space, `' '` as the fill character)
- `calign(number)` (center aligns text using specified argument as the width and an empty space, `' '` as the fill character)

### Conditional attributes

Say you have these logs:

```
level=INFO msg="some info"
level=ERROR msg="some error"
```

You use the following capture group to grab the log severity level:

```
'^level=(?<lvl>\w+)'
```

Depending on if it's `ERROR` or `INFO`, we may want to apply different colors. To achieve this affect, we can apply a `?` operator like so:

```bash
grits -p '^level=(?<lvl>\w+)' -t '${(?red("ERROR")|?cyan("INFO")):lvl}'
```

This will ensure that `ERROR` is printed with a red foreground while `INFO` is printed with a cyan foreground. The `?` operator modifies attributes
such that its first argument is a regular expression. If it's first argument matches the result of the capture group, then that attribute is applied.

The `?` operator is always prepended onto the attribute name like so: `?red`, `?cyan`, etc..

Here is another example of conditional attributes being applied in a case-insensitive manner using a regular expression instead of a string literal:

```bash
grits -p '^level=(?<lvl>\w+)' -t '${(?red("(?i)error")|?cyan("(?i)info")):lvl}'
```

### Filtering

If you want the result of a template transformation to show only if certain anchors have a corresponding match, then you can make use of the `!` operator
which is always placed immediate after the opening `{` of an anchor.

Say you have the following log lines:

```
level=INFO msg="some info"
level=ERROR msg="some error"
```

Let's say you want to filter only the `INFO` logs, everything else should be ignored. You can accomplish this using the operator `!` like so:

```bash
grits -p '^level=(?<lvl>INFO)' -t '${!lvl}'
```

What this effectively accomplishes is that only logs that have a match for the `lvl` capture group will be transformed. Here is what filtering
looks like with attributes:

```bash
grits -p '^level=(?<lvl>INFO)' -t '${!(red|bold):lvl}'
```

### Other examples

1. Multi-file processing:

```bash
grits -p 'sysctl=(?<sysctl>.*)'` -t 'sysctl output: {sysctl}' file1 file2
```

2. Piping:

```bash
docker logs -f 93670ea0964c | grits -p 'log_level=info(?<log>.*)' -t 'INFO LOG: {log}'
```

3. Attributes, default values, and multiple regular expressions:

```bash
kubectl logs -f -n foo -l app=bar | grits \
     -p '^kernel:(?<kern>.*)' \
     -p '^sysctl:(?<sys>.*)' \
     -t 'kernel={(cyan):kern || \"NONE\"} sysctl={(magenta):sys || \"NONE\"}'
```

## Completions

Completions for supported shells can be generated using `grits --completions <SHELl>`. Consult your shell's documentation
for how to setup completions. For `zsh`, completions are bootstrapped like so:

```bash
grits --completions zsh > ~/.oh-my-zsh/completions/_grits
```

## Colorization

`grits` follows the informal [NO_COLOR](https://no-color.org/) standard. Setting `NO_COLOR` to a non-blank value will disable output colorization.
If stdout is not a terminal, colorization is automatically disabled.
