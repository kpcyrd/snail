use errors::Result;

use kuchiki;
use kuchiki::traits::TendrilSink;
use std::collections::HashMap;
use hlua::AnyLuaValue;
use structs::LuaMap;


#[derive(Debug, PartialEq)]
pub struct Element {
    pub attrs: HashMap<String, String>,
    pub text: String,
    pub html: String,
}

impl Into<AnyLuaValue> for Element {
    fn into(self) -> AnyLuaValue {
        let mut map = LuaMap::new();

        map.insert_str("text", self.text);
        map.insert("attrs", LuaMap::from(self.attrs));
        map.insert_str("html", self.html);

        map.into()
    }
}

fn transform_element(entry: &kuchiki::NodeDataRef<kuchiki::ElementData>) -> Element {
    let text = entry.text_contents();
    let as_node = entry.as_node();

    let mut attrs: HashMap<String, String> = HashMap::new();

    if let Some(element) = as_node.as_element() {
        for (k, v) in &element.attributes.borrow().map {
            attrs.insert(k.local.to_string(), v.value.clone());
        }
    }

    let mut html = Vec::new();
    let html = match as_node.serialize(&mut html) {
        Ok(_) => String::from_utf8_lossy(&html).to_string(),
        Err(_) => {
            debug!("html serialize failed");
            String::new()
        },
    };

    Element {
        attrs,
        text,
        html,
    }
}

pub fn html_select(html: &str, selector: &str) -> Result<Element> {
    let doc = kuchiki::parse_html().one(html);
    match doc.select_first(selector) {
        Ok(x) => Ok(transform_element(&x)),
        Err(_) => bail!("css selector failed"),
    }
}

pub fn html_select_list(html: &str, selector: &str) -> Result<Vec<Element>> {
    let doc = kuchiki::parse_html().one(html);

    match doc.select(selector) {
        Ok(x) => Ok(x.into_iter().map(|x| transform_element(&x)).collect()),
        Err(_) => bail!("css selector failed"),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_select() {
        let elems = html_select(r#"<html><div id="yey">content</div></html>"#, "#yey").unwrap();
        assert_eq!(elems,
            Element {
                attrs: vec![(String::from("id"), String::from("yey"))].into_iter().collect(),
                text: "content".into(),
                html: r#"<div id="yey">content</div>"#.into(),
            }
        );
    }

    #[test]
    fn test_html_select_list() {
        let elems = html_select_list(r#"<html><div id="yey">content</div></html>"#, "#yey").unwrap();
        assert_eq!(elems, vec![
            Element {
                attrs: vec![(String::from("id"), String::from("yey"))].into_iter().collect(),
                text: "content".into(),
                html: r#"<div id="yey">content</div>"#.into(),
            }
        ]);
    }
}
