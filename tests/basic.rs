use roff::parse_to_json;

const SAMPLE: &str = r#"
.Dd August 31, 2020
.Dt LS 1
.Os
.Sh NAME
.Nm ls
.Nd list directory contents
.Sh DESCRIPTION
Text A
.Pp
Text B
.Sh OPTIONS
.Bl -tag -width indent
.It -a
All files
.It -l
Long format
.El
"#;

#[test]
fn parse_sample() {
    let v = parse_to_json(SAMPLE);
    let name = v.get("name").unwrap().as_str().unwrap();
    let section = v.get("section").unwrap().as_str().unwrap();
    assert_eq!(name, "ls");
    assert_eq!(section, "1");
    let sections = v.get("sections").unwrap().as_array().unwrap();
    assert_eq!(sections.len(), 3);
    let desc = &sections[1];
    let text = desc.get("text").unwrap().as_str().unwrap();
    assert!(text.contains("Text A"));
    assert!(text.contains("Text B"));
    let opts = &sections[2];
    let items = opts.get("items").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 2);
}

const NESTED: &str = r#"
.Dd June 26, 2026
.Dt TOOL 1
.Os
.Sh NAME
.Nm tool
.Nd nested list demo
.Sh OPTIONS
.Bl -tag -width indent
.It -a
Archive mode.
.Bl -tag -width indent
.It --fast
Fast algorithm.
.It --safe
Safety checks.
.El
.It -l
Long format.
.El
"#;

/// Nested `.Bl` lists must preserve depth: the inner `.El` may NOT close the
/// outer list, so depths come out as [0, 1, 1, 0]. (regression for #25)
#[test]
fn parse_nested_list_depths() {
    let v = parse_to_json(NESTED);
    let opts = v
        .get("sections")
        .and_then(|s| s.as_array())
        .unwrap()
        .iter()
        .find(|s| s.get("title").and_then(|t| t.as_str()) == Some("OPTIONS"))
        .expect("OPTIONS section");
    let items = opts.get("items").unwrap().as_array().unwrap();

    // -a, --fast, --safe, -l
    assert_eq!(items.len(), 4);

    let depths: Vec<u64> = items
        .iter()
        .map(|it| it.get("depth").and_then(|d| d.as_u64()).unwrap())
        .collect();
    assert_eq!(
        depths,
        vec![0, 1, 1, 0],
        "inner .El must not close outer list"
    );

    // schema: every item is an object with tag + body (not a flat string)
    for it in items {
        assert!(it.is_object(), "items must be objects, got {it:?}");
        assert!(it.get("tag").is_some());
        assert!(it.get("body").is_some());
    }
}
