use super::{
    parse::attr::{Attribute, AttributeKind},
    OutputTemplate,
};
use crossterm::style::Stylize;
use std::collections::HashMap;

#[test]
fn test_output_template_basic() {
    let template = "log={foo} out={bar} baz";
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 5);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);
    interpolation_map.insert("bar", vec!["bar_value"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "log=foo_value out=bar_value baz")
}

#[test]
fn test_output_template_no_match() {
    let template = "log={foo} out={bar} baz";
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 5);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "log=foo_value out= baz")
}

#[test]
fn test_output_template_default() {
    let template = r#"log={foo} out={bar || "foobaz"} baz"#;
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 5);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "log=foo_value out=foobaz baz")
}

#[test]
fn test_output_template_indexes() {
    let template = r#"log={foo} out={bar[1]}"#;
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 4);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);
    interpolation_map.insert("bar", vec!["bar_value_1", "bar_value_2"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "log=foo_value out=bar_value_2")
}
#[test]
fn test_output_template_indexes_and_defaults() {
    let template = r#"{foo[1]||bar[1]}"#;
    let out = OutputTemplate::parse(template).unwrap();
    println!("{out:?}");
    assert_eq!(out.targets.len(), 1);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);
    interpolation_map.insert("bar", vec!["bar_value_1", "bar_value_2"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "bar_value_2")
}

#[test]
fn test_output_template_attributes() {
    let template = r#"{(red|bold):foo[1]||bar[1]}"#;
    let out = OutputTemplate::parse(template).unwrap();
    println!("{out:?}");
    assert_eq!(out.targets.len(), 1);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);
    interpolation_map.insert("bar", vec!["bar_value_1", "bar_value_2"]);

    let resultant = out.transform(&interpolation_map);
    let expected = Attribute::apply(
        "bar_value_2",
        &[
            Attribute {
                kind: AttributeKind::Red,
                must_match: None,
            },
            Attribute {
                kind: AttributeKind::Bold,
                must_match: None,
            },
        ],
    );
    assert_eq!(resultant, format!("{expected}"));
}

#[test]
fn test_output_template_required() {
    let template = "log={!foo} out={bar} baz";
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 5);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec![]);
    interpolation_map.insert("bar", vec!["bar_value"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(
        resultant, "",
        "foo doesn't have any matches so string should come back empty"
    );
}

#[test]
fn test_output_template_conditional_attr() {
    let template = "severity={(?red('(?i)error')|?cyan('(?i)info')):lvl}";
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 2);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("lvl", vec!["INFO"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, format!("severity={}", "INFO".cyan()));

    interpolation_map.insert("lvl", vec!["ERROR"]);
    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, format!("severity={}", "ERROR".red()));
}

#[test]
fn test_output_template_alignment() {
    let template = "{(lalign(9)):foo}";
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 1);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["bar"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "bar      ");

    let template = "{(lalign(9)|red):foo}";
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 1);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["bar"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "bar      ".red().to_string());

    let template = "{(red|lalign(9)):foo}";
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 1);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["bar"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "bar      ".red().to_string());
}
