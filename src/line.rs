use crate::{
    cli::ProcessArgs,
    scanner::{MultiFileScanner, StdinScanner},
    TtyContext,
};
use anyhow::{format_err, Context, Result};
use regex::Regex;
use std::collections::HashMap;

const OUTPUT_TEMPLATE_RE: &str = r#"\$\{(?<anchor>[_a-zA-Z0-9]+)(\[(?<index>\w+)\])*\}"#;

/// Warn users that a named capture in a subsequent pattern will override same name in previous.
/// Named capture groups should not have similar names across patterns.
pub fn run(tty: &mut TtyContext, args: &ProcessArgs) -> Result<()> {
    let ProcessArgs {
        pattern,
        output_template,
        files,
    } = args;

    let output_line_re = Regex::new(OUTPUT_TEMPLATE_RE).unwrap();
    if !output_line_re.is_match(output_template) {
        return Err(format_err!("provided output template string is not valid"));
    }

    let mut regexes = Vec::new();
    for pat in pattern {
        let re = Regex::new(pat)
            .with_context(|| format!("encountered invalid regular expression in pattern: {pat}"))?;
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

    for line in scanner {
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

        // TODO perform substitution

        println!("{line}");
    }

    Ok(())
}

#[test]
fn test_output_template() {
    let re = Regex::new(r#"\$\{\s*(?<anchor>[_a-zA-Z0-9]+)(\[(?<index>\d+)\])*(\s*\|\|\s*['"](?<default>[^'"]+)['"])*\s*\}"#).unwrap();
    let template =
        r#"${level[0]} | ${level[1]} | ${level[2] || 'foo'} | ${level[3] || "foobar"} | ${msg}"#;

    for capture_matches in re.captures_iter(template) {
        println!("{:?}", capture_matches.name("anchor"));
        println!("{:?}", capture_matches.name("index"));
        println!("{:?}", capture_matches.name("default"));
    }
    assert!(re.is_match(template))
}

#[test]
fn test_poc() {
    let re = Regex::new(r"\$\{([_a-zA-Z0-9]+)\}").unwrap();
    let target = "level=info msg='startup ok' level=warn";
    let template = "${level} | ${msg}";

    let regexes = vec![
        Regex::new(r"level=(?<level>\w+)").unwrap(),
        Regex::new(r"msg='(?<msg>[a-zA-Z ]+)'").unwrap(),
    ];

    let mut captures_map: HashMap<String, Vec<String>> = HashMap::default();

    for regex in regexes {
        for capture_match in regex.captures_iter(target) {
            for capture_name in regex.capture_names().filter_map(|n| n.map(String::from)) {
                let Some(val) = capture_match.name(&capture_name) else {
                    continue;
                };
                captures_map
                    .entry(capture_name)
                    .and_modify(|c| c.push(val.as_str().to_string()))
                    .or_insert_with(|| vec![val.as_str().to_string()]);
            }
        }
    }
}
