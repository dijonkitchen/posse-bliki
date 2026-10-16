//! A strict XML 1.0 well-formedness checker and tree builder for the RSS feed
//! and sitemap. It replaces feedparser's "bozo" check (ADR 2026-10-11-rust-only-toolchain): one root
//! element, matched tags, quoted unique attributes, only the five predefined
//! entities plus numeric references, no stray `<` or `&`.

use super::html::{Element, Node};

fn decode(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i + 1..];
        let end = rest.find(';').ok_or("'&' without ';'")?;
        let name = &rest[..end];
        let c = match name {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let num = name.strip_prefix('#').ok_or(format!("undefined entity &{name};"))?;
                let cp = match num.strip_prefix('x') {
                    Some(hex) => u32::from_str_radix(hex, 16),
                    None => num.parse(),
                }
                .map_err(|_| format!("bad reference &{name};"))?;
                char::from_u32(cp).ok_or(format!("bad code point &{name};"))?
            }
        };
        out.push(c);
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || "_-.:".contains(c)
}

/// Parse `src` into its root element, or say why it is not well-formed.
pub fn parse(src: &str) -> Result<Element, String> {
    let mut rest = src.strip_prefix('\u{feff}').unwrap_or(src);
    if let Some(after) = rest.strip_prefix("<?xml") {
        rest = &after[after.find("?>").ok_or("unterminated XML declaration")? + 2..];
    }
    let mut stack: Vec<Element> = vec![Element {
        name: "#document".into(),
        attrs: vec![],
        children: vec![],
    }];
    let mut roots = 0;
    while !rest.is_empty() {
        let lt = rest.find('<').unwrap_or(rest.len());
        let text = &rest[..lt];
        if stack.len() == 1 {
            if !text.trim().is_empty() {
                return Err(format!("text outside the root element: {:?}", text.trim()));
            }
        } else if !text.is_empty() {
            if text.contains("]]>") {
                return Err("']]>' in text".into());
            }
            stack.last_mut().unwrap().children.push(Node::Text(decode(text)?));
        }
        rest = &rest[lt..];
        if rest.is_empty() {
            break;
        }
        if let Some(after) = rest.strip_prefix("<!--") {
            rest = &after[after.find("-->").ok_or("unterminated comment")? + 3..];
        } else if let Some(after) = rest.strip_prefix("<![CDATA[") {
            let end = after.find("]]>").ok_or("unterminated CDATA")?;
            if stack.len() == 1 {
                return Err("CDATA outside the root element".into());
            }
            stack
                .last_mut()
                .unwrap()
                .children
                .push(Node::Text(after[..end].to_string()));
            rest = &after[end + 3..];
        } else if let Some(after) = rest.strip_prefix("</") {
            let len = after.find(|c| !is_name_char(c)).unwrap_or(after.len());
            let name = &after[..len];
            let tail = after[len..].trim_start();
            if !tail.starts_with('>') {
                return Err(format!("malformed </{name}>"));
            }
            rest = &tail[1..];
            if stack.len() == 1 || stack.last().unwrap().name != name {
                return Err(format!("</{name}> does not match <{}>", stack.last().unwrap().name));
            }
            let el = stack.pop().unwrap();
            stack.last_mut().unwrap().children.push(Node::Element(el));
        } else {
            let after = &rest[1..];
            let len = after.find(|c| !is_name_char(c)).unwrap_or(after.len());
            if len == 0 {
                return Err("'<' that does not start a tag".into());
            }
            let mut el = Element {
                name: after[..len].to_string(),
                attrs: vec![],
                children: vec![],
            };
            let mut tail = &after[len..];
            let self_closing;
            loop {
                let trimmed = tail.trim_start();
                let had_ws = trimmed.len() < tail.len();
                tail = trimmed;
                if let Some(t) = tail.strip_prefix("/>") {
                    tail = t;
                    self_closing = true;
                    break;
                }
                if let Some(t) = tail.strip_prefix('>') {
                    tail = t;
                    self_closing = false;
                    break;
                }
                if !had_ws {
                    return Err(format!("<{}>: malformed attributes", el.name));
                }
                let n = tail.find(|c| !is_name_char(c)).unwrap_or(tail.len());
                let name = tail[..n].to_string();
                let t = tail[n..]
                    .trim_start()
                    .strip_prefix('=')
                    .ok_or(format!("<{}>: {name} has no value", el.name))?;
                let t = t.trim_start();
                let q = t
                    .chars()
                    .next()
                    .filter(|c| *c == '"' || *c == '\'')
                    .ok_or("unquoted attribute value")?;
                let end = t[1..].find(q).ok_or("unterminated attribute value")?;
                let raw = &t[1..end + 1];
                if raw.contains('<') {
                    return Err(format!("'<' in attribute {name}"));
                }
                if el.attrs.iter().any(|(k, _)| *k == name) {
                    return Err(format!("<{}>: duplicate attribute {name}", el.name));
                }
                el.attrs.push((name, decode(raw)?));
                tail = &t[end + 2..];
            }
            rest = tail;
            if stack.len() == 1 {
                roots += 1;
                if roots > 1 {
                    return Err("more than one root element".into());
                }
            }
            if self_closing {
                stack.last_mut().unwrap().children.push(Node::Element(el));
            } else {
                stack.push(el);
            }
        }
    }
    if stack.len() > 1 {
        return Err(format!("unclosed <{}>", stack.last().unwrap().name));
    }
    stack
        .pop()
        .unwrap()
        .elements()
        .next()
        .cloned()
        .ok_or_else(|| "no root element".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_well_formedness() {
        let root = parse("<?xml version=\"1.0\"?>\n<a x=\"1\"><b>&amp;&#38;<![CDATA[<i>]]></b><c/></a>\n").unwrap();
        assert_eq!(root.child("b").unwrap().text(), "&&<i>");
        for bad in [
            "<a><b></a>",
            "<a>&nbsp;</a>",
            "<a>1 < 2</a>",
            "<a x=1/>",
            "<a></a><b></b>",
            "<a>",
        ] {
            assert!(parse(bad).is_err(), "accepted {bad:?}");
        }
    }
}
