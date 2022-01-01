use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Node>,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.node_type {
            NodeType::Text(text) => write!(f, "{}", text),
            NodeType::Element(elem) => write!(
                f,
                "<{}>{}</{}>",
                elem.tag_name,
                {
                    let mut s = String::from("");
                    self.children.as_slice().iter().for_each(|node| {
                        s = format!("{}{}", s, node);
                    });
                    s
                },
                elem.tag_name
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Text(text) => write!(f, "{}", text),
            NodeType::Element(elem) => write!(f, "<{}></{}>", elem.tag_name, elem.tag_name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttrMap,
}

pub type AttrMap = HashMap<String, String>;

pub fn text(data: String) -> Node {
    Node {
        node_type: NodeType::Text(data),
        children: Vec::new(),
    }
}

pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs,
        }),
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_simple() {
        let node = elem(
            String::from("div"),
            HashMap::new(),
            vec![
                elem(
                    String::from("h1"),
                    HashMap::new(),
                    vec![text(String::from("h1 text"))],
                ),
                elem(
                    String::from("h2"),
                    HashMap::new(),
                    vec![text(String::from("h2 text"))],
                ),
                elem(
                    String::from("h3"),
                    HashMap::new(),
                    vec![text(String::from("h3 text"))],
                ),
            ],
        );
        assert_eq!(
            "<div><h1>h1 text</h1><h2>h2 text</h2><h3>h3 text</h3></div>",
            format!("{}", node)
        );
    }
}
