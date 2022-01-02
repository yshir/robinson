use std::collections::{HashMap, HashSet};

use crate::dom;

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
                "<{}{}>{}</{}>",
                elem.tag_name,
                {
                    let mut s = String::from("");
                    let mut attrs = elem.attributes.iter().collect::<Vec<_>>();
                    attrs.sort();
                    for (name, value) in attrs {
                        s = format!("{} {}=\"{}\"", s, name, value);
                    }
                    s
                },
                {
                    let mut s = String::from("");
                    for node in &self.children {
                        s = format!("{}{}", s, node);
                    }
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

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(class_list) => class_list.split(' ').collect(),
            None => HashSet::new(),
        }
    }
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

pub struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    // Read the current character without consuming it
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // Return true if the next characters start with the given string
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // Return true if all input is consumed
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    // Return the current character, and advance self.pos to the next character
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    // Consume characters until `test` returns false
    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char())
        }
        result
    }

    // Consume and discard zero or more whitespace characters
    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_ascii_whitespace());
    }

    // Parse a tag or attribute name
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }

    // Parse a single name="value" pair.
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        (name, value)
    }

    // Parse q quoted value
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }

    // Parse a list of name="value" pairs, separated by whitespace
    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        attributes
    }

    // Parse a single node
    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    // Parse a text node
    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }

    // Parse a single element, including its open tag, contents, and closing tag
    fn parse_element(&mut self) -> dom::Node {
        // Opening tag
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        // Contents
        let children = self.parse_nodes();

        // Closing tag
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        dom::elem(tag_name, attrs, children)
    }

    // Parse a sequence of sibling nodes
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    // Parse an HTML document and return the root element
    pub fn parse(source: String) -> dom::Node {
        let mut nodes = Parser {
            pos: 0,
            input: source,
        }
        .parse_nodes();

        // If the document contains a root element, just return it.
        // Otherwise, create one.
        if nodes.len() == 1 {
            nodes.swap_remove(0)
        } else {
            dom::elem("html".to_string(), HashMap::new(), nodes)
        }
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

    #[test]
    fn display_attr() {
        let node = elem(
            String::from("div"),
            HashMap::from([
                (String::from("a"), String::from("b")),
                (String::from("c"), String::from("d")),
            ]),
            Vec::new(),
        );
        assert_eq!("<div a=\"b\" c=\"d\"></div>", format!("{}", node));
    }

    #[test]
    fn parse_simple() {
        let node = Parser::parse(
            "
<p>p1</p>
<p>p2</p>
<p a=\"b\">p3</p>
        "
            .to_string(),
        );

        assert_eq!(
            "<html><p>p1</p><p>p2</p><p a=\"b\">p3</p></html>",
            format!("{}", node)
        );
    }
}
