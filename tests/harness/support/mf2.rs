//! A microformats2 parser covering what spec/output-contract.md and
//! spec/indieweb.md check: root `h-*` items, `p-`/`u-`/`dt-`/`e-`
//! properties, nested items as property values or children, and `rels`.
//! It replaces mf2py (ADR 0010). Implied properties and value-class parsing
//! are left out: the build always marks properties explicitly.

use super::html::{Element, Node};

#[derive(Debug, Clone)]
pub enum Value {
    Text(String),
    Item(Item),
}

#[derive(Debug, Clone, Default)]
pub struct Item {
    pub types: Vec<String>,
    pub props: Vec<(String, Vec<Value>)>,
    pub children: Vec<Item>,
}

impl Item {
    pub fn has_type(&self, t: &str) -> bool {
        self.types.iter().any(|x| x == t)
    }

    /// The values of property `name`; empty when absent.
    pub fn prop(&self, name: &str) -> &[Value] {
        self.props
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_slice())
            .unwrap_or(&[])
    }

    /// The text values of property `name`.
    pub fn texts(&self, name: &str) -> Vec<&str> {
        self.prop(name)
            .iter()
            .filter_map(|v| match v {
                Value::Text(t) => Some(t.as_str()),
                Value::Item(_) => None,
            })
            .collect()
    }

    fn add(&mut self, name: &str, v: Value) {
        match self.props.iter_mut().find(|(k, _)| k == name) {
            Some((_, vs)) => vs.push(v),
            None => self.props.push((name.to_string(), vec![v])),
        }
    }
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub items: Vec<Item>,
    pub rels: Vec<(String, Vec<String>)>,
}

impl Parsed {
    pub fn items_of_type(&self, t: &str) -> Vec<&Item> {
        self.items.iter().filter(|i| i.has_type(t)).collect()
    }

    pub fn rel(&self, name: &str) -> &[String] {
        self.rels
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_slice())
            .unwrap_or(&[])
    }
}

fn root_types(el: &Element) -> Vec<String> {
    el.classes()
        .into_iter()
        .filter(|c| c.len() > 2 && c.starts_with("h-"))
        .map(String::from)
        .collect()
}

/// `(prefix, name)` for each property class on `el`.
fn prop_classes(el: &Element) -> Vec<(&str, &str)> {
    el.classes()
        .into_iter()
        .filter_map(|c| {
            ["p-", "u-", "dt-", "e-"]
                .into_iter()
                .find(|p| c.starts_with(p) && c.len() > p.len())
                .map(|p| (p, &c[p.len()..]))
        })
        .collect()
}

fn value(prefix: &str, el: &Element) -> String {
    let attr = match (prefix, el.name.as_str()) {
        ("u-", "a" | "area" | "link") => el.attr("href"),
        ("u-", "img" | "audio" | "video" | "source") => el.attr("src"),
        ("dt-", "time" | "ins" | "del") => el.attr("datetime"),
        ("p-", "img" | "area") => el.attr("alt"),
        _ => None,
    };
    attr.map(String::from).unwrap_or_else(|| el.text().trim().to_string())
}

fn parse_item(el: &Element) -> Item {
    let mut item = Item {
        types: root_types(el),
        ..Item::default()
    };
    for c in el.elements() {
        collect(c, &mut item);
    }
    item
}

fn collect(el: &Element, item: &mut Item) {
    let props = prop_classes(el);
    if !root_types(el).is_empty() {
        let sub = parse_item(el);
        if props.is_empty() {
            item.children.push(sub);
        } else {
            for (_, name) in props {
                item.add(name, Value::Item(sub.clone()));
            }
        }
        return;
    }
    for (prefix, name) in props {
        item.add(name, Value::Text(value(prefix, el)));
    }
    for c in el.elements() {
        collect(c, item);
    }
}

pub fn parse(doc: &Element) -> Parsed {
    let mut out = Parsed::default();
    for el in doc.descendants() {
        let Some(href) = el.attr("href").filter(|_| el.name == "a" || el.name == "link") else {
            continue;
        };
        for r in el.attr("rel").unwrap_or("").split_ascii_whitespace() {
            match out.rels.iter_mut().find(|(k, _)| k == r) {
                Some((_, hrefs)) if hrefs.iter().any(|h| h == href) => {}
                Some((_, hrefs)) => hrefs.push(href.to_string()),
                None => out.rels.push((r.to_string(), vec![href.to_string()])),
            }
        }
    }
    fn walk(el: &Element, items: &mut Vec<Item>) {
        if !root_types(el).is_empty() {
            items.push(parse_item(el));
            return;
        }
        for c in &el.children {
            if let Node::Element(e) = c {
                walk(e, items);
            }
        }
    }
    walk(doc, &mut out.items);
    out
}

#[cfg(test)]
mod tests {
    use super::super::html;
    use super::*;

    #[test]
    fn nested_items_become_property_values() {
        let doc = html::parse(
            "<!DOCTYPE html><html><head><title>t</title><link rel=\"me\" href=\"https://a.example/\"></head><body>\
             <article class=\"h-entry\"><h1 class=\"p-name\">Hi</h1>\
             <a class=\"u-url\" href=\"/x/\"><time class=\"dt-published\" datetime=\"2026-01-02\">Jan</time></a>\
             <span class=\"p-author h-card\"><a class=\"u-url p-name\" rel=\"author\" href=\"/\">Me</a></span>\
             <div class=\"e-content\"><p>Body</p></div></article></body></html>",
        )
        .unwrap();
        let parsed = parse(&doc);
        let entry = parsed.items_of_type("h-entry")[0];
        assert_eq!(entry.texts("name"), ["Hi"]);
        assert_eq!(entry.texts("url"), ["/x/"]);
        assert_eq!(entry.texts("published"), ["2026-01-02"]);
        assert_eq!(entry.texts("content"), ["Body"]);
        match &entry.prop("author")[0] {
            Value::Item(card) => assert!(card.has_type("h-card") && card.texts("name") == ["Me"]),
            other => panic!("author is {other:?}"),
        }
        assert_eq!(parsed.rel("me"), ["https://a.example/"]);
        assert_eq!(parsed.rel("author"), ["/"]);
    }
}
