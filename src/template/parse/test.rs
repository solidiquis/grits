use super::{parse, DefaultValue};

//${(red|green|bold):foo}

#[test]
fn test_parse_plain() {
    let template_string = "output=${log}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);

    let anchor = &anchors[0];
    assert_eq!(&anchor.name, "log");
    assert_eq!(anchor.start, 7);
    assert_eq!(anchor.end, 13);
    assert_eq!(anchor.index, None);
}

#[test]
fn test_parse_plain_whitespace() {
    let template_string = "\toutput=${\tlog\n}   ";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);
    let anchor = &anchors[0];
    assert_eq!(&anchor.name, "log");
    assert_eq!(anchor.start, 8);
    assert_eq!(anchor.end, 16);
    assert_eq!(anchor.index, None);
}

#[test]
fn test_parse_escape() {
    let template_string = r"\$ output=${log}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);

    let anchor = &anchors[0];
    assert_eq!(&anchor.name, "log");
    assert_eq!(anchor.start, 10);
    assert_eq!(anchor.end, 16);
    assert_eq!(anchor.index, None);

    let template_string = r"\$ \\ output=${log}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);

    let anchor = &anchors[0];
    assert_eq!(&anchor.name, "log");
    assert_eq!(anchor.start, 13);
    assert_eq!(anchor.end, 19);
    assert_eq!(anchor.index, None);
}

#[test]
fn test_parse_index() {
    let template_string = "primary=${log[0]} secondary=${log[102]}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 2);

    let anchor = &anchors[0];
    assert_eq!(&anchor.name, "log");
    assert_eq!(anchor.start, 8);
    assert_eq!(anchor.end, 17);
    assert_eq!(anchor.index, Some(0));

    let anchor = &anchors[1];
    assert_eq!(&anchor.name, "log");
    assert_eq!(anchor.start, 28);
    assert_eq!(anchor.end, 39);
    assert_eq!(anchor.index, Some(102));
}

#[test]
fn test_parse_index_errors() {
    let template_string = "primary=${[0]}";
    let anchors = parse(template_string);
    assert!(anchors.is_err_and(|e| e.to_string().contains("Invalid index operation")));

    let template_string = "primary=${  \t  [0]}";
    let anchors = parse(template_string);
    assert!(anchors.is_err_and(|e| e.to_string().contains("Invalid index operation")));

    let template_string = "primary=${foobar [0]}";
    let anchors = parse(template_string);
    assert!(anchors.is_err_and(|e| e.to_string().contains("Invalid index operation")));
}

#[test]
fn test_default_literal_parse_single_value() {
    let template_string = "primary=${foo || 'bar'}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);

    let defaults = &anchors[0].defaults;
    assert_eq!(defaults.len(), 1);

    let DefaultValue::Literal(val) = &defaults[0] else {
        panic!("expected default to be a literal");
    };
    assert_eq!(val, "bar");
}

#[test]
fn test_default_literal_parse_multi_value() {
    let template_string = "primary=${foo || 'bar' || 'baz'}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);

    let defaults = &anchors[0].defaults;
    assert_eq!(defaults.len(), 2);

    let expected_literals = ["bar", "baz"];

    for default in defaults {
        let DefaultValue::Literal(val) = default else {
            panic!("expected default to be a literal");
        };
        assert!(expected_literals.contains(&val.as_str()));
    }
}

#[test]
fn test_default_anchor() {
    let template_string = "primary=${foo||bar}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);
    let anchor = &anchors[0];
    let default_values = &anchor.defaults;
    assert_eq!(default_values.len(), 1);
    let DefaultValue::Anchor { name, index } = &default_values[0] else {
        panic!("expected default value to be anchor");
    };
    assert_eq!(name, "bar");
    assert!(index.is_none());
}

#[test]
fn test_default_anchor_whitespace() {
    let template_string = "primary=${foo || bar}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);
    let anchor = &anchors[0];
    let default_values = &anchor.defaults;
    assert_eq!(default_values.len(), 1);
    let DefaultValue::Anchor { name, index } = &default_values[0] else {
        panic!("expected default value to be anchor");
    };
    assert_eq!(name, "bar");
    assert!(index.is_none());
}

#[test]
fn test_default_anchor_indexes() {
    if std::env::var("RUST_LOG").is_ok() {
        env_logger::init();
    }
    let template_string = "primary=${foo || bar[0]}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);
    let anchor = &anchors[0];
    let default_values = &anchor.defaults;
    assert_eq!(default_values.len(), 1);
    let DefaultValue::Anchor { name, index } = &default_values[0] else {
        panic!("expected default value to be anchor");
    };
    assert_eq!(name, "bar");
    assert!(index.is_some_and(|i| i == 0));
}

#[test]
fn test_default_anchor_multi_value() {
    let template_string = "primary=${foo || bar || baz}";
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);
    let anchor = &anchors[0];
    let default_values = &anchor.defaults;
    assert_eq!(default_values.len(), 2);

    for default_value in default_values {
        let DefaultValue::Anchor { name, index } = default_value else {
            panic!("expected default anchor");
        };
        assert!(name == "bar" || name == "baz");
        assert!(index.is_none());
    }
}

#[test]
fn test_default_values_multi_value_with_indexes() {
    let template_string = r#"primary=${foo[3] || bar[0] || "baz"}"#;
    let anchors = parse(template_string).unwrap();
    assert_eq!(anchors.len(), 1);
    let anchor = &anchors[0];
    assert!(anchor.index.is_some_and(|i| i == 3));
    let default_values = &anchor.defaults;
    assert_eq!(default_values.len(), 2);

    for default_value in default_values {
        match default_value {
            DefaultValue::Anchor { name, index } => {
                assert!(name == "bar");
                assert!(index.is_some_and(|i| i == 0));
            }
            DefaultValue::Literal(val) => {
                assert!(val == "baz");
            }
        }
    }
}
