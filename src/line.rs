use crate::{
    scanner::{MultiFileScanner, StdinScanner},
    template::OutputTemplate,
    tty::init_output_writer,
    Cli, TtyContext,
};
use anyhow::{format_err, Context, Result};
use regex::Regex;
use std::collections::HashMap;

/// Process input lines from files or standard input.
pub fn process_lines(tty: &mut TtyContext, args: &Cli) -> Result<()> {
    let Cli {
        pattern,
        template,
        files,
        line_buffered,
        require,
        ..
    } = args;

    let filters = require.split(",").map(str::trim).collect::<Vec<_>>();

    let mut templates = Vec::with_capacity(template.len());
    for templ in template {
        templates.push(OutputTemplate::parse(templ)?);
    }

    let mut regexes = Vec::new();
    for pat in pattern {
        let re = Regex::new(pat).with_context(|| format!("encountered invalid regular expression: {pat}"))?;
        regexes.push(re);
    }

    let mut regex_with_cached_capture_names = Vec::with_capacity(regexes.len());
    let mut captures_map: HashMap<&str, Vec<&str>> = HashMap::new();

    for regex in regexes.iter() {
        let capnames = regex.capture_names().flatten().collect::<Vec<_>>();

        for capture_name in &capnames {
            captures_map.entry(capture_name).or_default();
        }
        regex_with_cached_capture_names.push((regex, capnames));
    }

    if captures_map.is_empty() {
        return Err(format_err!(
            "none of the provided patterns contained named capture groups"
        ));
    }

    let scanner = {
        if files.is_empty() {
            StdinScanner::init()
        } else {
            MultiFileScanner::init(files)?
        }
    };

    let mut writer = init_output_writer(tty, *line_buffered);

    'outer: for line in scanner {
        // Each iteration starts with a fresh captures map. Doing it this way the lifetime of the
        // new captures map contain the lifetime of `line`, allowing us to work with a `Vec<&str>`
        // as opposed to `Vec<String>`. There's no telling how many matches there could possibly be
        // per line so we're optimizing for minimal string allocations.
        let mut captures_map = captures_map.clone();

        // populate each key of the captures map
        for (regex, capture_names) in &regex_with_cached_capture_names {
            for capture_match in regex.captures_iter(&line) {
                for capture_name in capture_names {
                    let Some(val) = capture_match.name(capture_name) else {
                        continue;
                    };
                    captures_map
                        .entry(capture_name)
                        .and_modify(|c| c.push(val.as_str()))
                        .or_insert_with(|| vec![val.as_str()]);
                }
            }
        }
        for capture_name in &filters {
            if captures_map.get(capture_name).is_none_or(|c| c.is_empty()) {
                continue 'outer;
            }
        }
        for template in &templates {
            let output_line = template.transform(&captures_map);

            if output_line.is_empty() {
                continue;
            }
            writer.writeln(&output_line)?;
        }
    }
    Ok(())
}
