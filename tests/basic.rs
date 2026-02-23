use man_parser::parse_to_json;

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

