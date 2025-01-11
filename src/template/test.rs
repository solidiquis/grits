use super::{Attribute, OutputTemplate};
use std::collections::HashMap;

#[test]
fn test_output_template_basic() {
    let template = "log=${foo} out=${bar} baz";
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
    let template = "log=${foo} out=${bar} baz";
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 5);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "log=foo_value out= baz")
}

#[test]
fn test_output_template_default() {
    let template = r#"log=${foo} out=${bar || "foobaz"} baz"#;
    let out = OutputTemplate::parse(template).unwrap();
    assert_eq!(out.targets.len(), 5);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);

    let resultant = out.transform(&interpolation_map);
    assert_eq!(resultant, "log=foo_value out=foobaz baz")
}

#[test]
fn test_output_template_indexes() {
    let template = r#"log=${foo} out=${bar[1]}"#;
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
    let template = r#"${foo[1]||bar[1]}"#;
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
    let template = r#"${(red|bold):foo[1]||bar[1]}"#;
    let out = OutputTemplate::parse(template).unwrap();
    println!("{out:?}");
    assert_eq!(out.targets.len(), 1);

    let mut interpolation_map = HashMap::new();
    interpolation_map.insert("foo", vec!["foo_value"]);
    interpolation_map.insert("bar", vec!["bar_value_1", "bar_value_2"]);

    let resultant = out.transform(&interpolation_map);
    let expected = Attribute::apply("bar_value_2", &[Attribute::Red, Attribute::Bold]);
    assert_eq!(resultant, format!("{expected}"));
}
