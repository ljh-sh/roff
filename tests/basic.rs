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

const TP: &str = r#"
.Dd June 26, 2026
.Dt TOOL 1
.Os
.Sh OPTIONS
.TP
.B \-a
Do not ignore entries starting with a dot.
This includes hidden files too.
.TP
.B \-l
Use a long listing format.
"#;

/// `.TP`: the next line is the tag, subsequent lines join into the body, and
/// items must not bleed across `.TP` boundaries. (regression for #26)
#[test]
fn parse_tp_tag_body_separation() {
    let v = parse_to_json(TP);
    let opts = v
        .get("sections")
        .and_then(|s| s.as_array())
        .unwrap()
        .iter()
        .find(|s| s.get("title").and_then(|t| t.as_str()) == Some("OPTIONS"))
        .expect("OPTIONS section");
    let items = opts.get("items").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 2, "no bleed across .TP boundaries");

    let first = &items[0];
    let tag = first.get("tag").and_then(|t| t.as_str()).unwrap();
    let body = first.get("body").and_then(|b| b.as_str()).unwrap();
    assert!(tag.contains("-a"), "tag should hold the flag, got {tag:?}");
    assert!(
        !tag.contains("Do not ignore"),
        "tag must not include body text"
    );
    assert!(body.contains("Do not ignore"));
    assert!(
        body.contains("hidden files too"),
        "multi-line body must join into one item"
    );
}

#[test]
fn to_html_well_formed() {
    let html = roff::to_html(&parse_to_json(NESTED));
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<html lang=\"en\">"));
    assert!(html.contains("<meta charset=\"utf-8\">"));
    // front matter -> <meta> tags
    assert!(html.contains("<meta name=\"title\" content=\"TOOL\">"));
    assert!(html.contains("<meta name=\"name\" content=\"tool\">"));
    // document + section structure
    assert!(html.contains("<h1>TOOL</h1>"));
    assert!(html.contains("<h2>OPTIONS</h2>"));
    // tags render as <code>; nested item present
    assert!(html.contains("<li><code>"));
    assert!(html.contains("--fast"));
    assert!(html.trim_end().ends_with("</html>"));
}

#[test]
fn to_html_escapes_special_chars() {
    let src = ".Dt T 1\n.Sh NAME\n.Nm t\n.Nd a <b> tag & amp\n";
    let html = roff::to_html(&parse_to_json(src));
    assert!(
        !html.contains("<b> tag"),
        "raw < must be escaped, got: {html:?}"
    );
    assert!(html.contains("&lt;b&gt;"));
    assert!(html.contains("&amp; amp"));
}

/// `.so` cycles must not stack-overflow: an indirect cycle (a -> b -> a) is
/// detected and recorded in `source_skipped` instead of recursing forever. (#2)
#[test]
fn so_cycle_does_not_overflow() {
    use std::fs;
    let dir = std::env::temp_dir().join("roff-so-cycle-test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.1"), ".TH A 1\n.so b.1\n").unwrap();
    fs::write(dir.join("b.1"), ".TH B 1\n.so a.1\n").unwrap();

    let a = dir.join("a.1");
    let content = fs::read_to_string(&a).unwrap();
    let v = roff::parse_to_json_with_opts(&content, true, a.to_str());
    let skipped = v
        .get("source_skipped")
        .and_then(|s| s.as_array())
        .expect("cycle should be recorded in source_skipped");
    assert!(
        skipped
            .iter()
            .any(|s| s.as_str().unwrap_or("").contains("cycle")),
        "expected a cycle marker, got {skipped:?}"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// YAML front matter must escape `:` and `"` so a description like
/// `list: a "q" value` produces valid (quoted) YAML. (#4)
#[test]
fn to_markdown_frontmatter_escapes_yaml_special_chars() {
    let src = ".Dt T 1\n.Sh NAME\n.Nm t\n.Nd list: a \"q\" value\n";
    let md = roff::to_markdown(&roff::parse_to_json(src));
    let desc_line = md
        .lines()
        .find(|l| l.starts_with("description:"))
        .expect("description front matter");
    assert_eq!(
        desc_line, "description: \"list: a \\\"q\\\" value\"",
        "value must be quoted with internal quotes escaped"
    );
}

/// `env` is emitted as a proper YAML sequence, not a `VAR: true` mapping. (#4)
#[test]
fn to_markdown_env_is_yaml_sequence() {
    let src = ".Dt T 1\n.Sh NAME\n.Nm t\n.Nd demo\n.Sh ENVIRONMENT\n.Ev FOO\n.Ev BAR\n";
    let md = roff::to_markdown(&roff::parse_to_json(src));
    let env_block: Vec<&str> = md
        .lines()
        .skip_while(|l| !l.starts_with("env:"))
        .take_while(|l| l.starts_with("env:") || l.starts_with("  - "))
        .collect();
    assert_eq!(env_block[0], "env:");
    assert!(
        env_block[1..].iter().all(|l| l.starts_with("  - ")),
        "env items must be a sequence"
    );
}
