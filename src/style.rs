use std::collections::HashMap;

use crate::{
    css::{Rule, Selector, SimpleSelector, Specificity, StyleSheet, Value},
    dom::{ElementData, Node, NodeType},
};

// Map from CSS property names to values
pub type PropertyMap = HashMap<String, Value>;

// A node with associated style data
pub struct StyledNode<'a> {
    pub node: &'a Node, // pointer to a DOM node
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

pub enum Display {
    Inline,
    Block,
    None,
}

impl StyledNode<'_> {
    // Return the specified value of a property if it exists, otherwise `None`.
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    // The value of the `display` property (defaults to inline)
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    // Return specified value of property `name`, or property `fallback_name` if that doesn't exist
    // or value `default` if neither does
    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }
}

// Apply a stylesheet to an entire DOM tree, returning a StyleNode tree
pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a StyleSheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new(),
        },
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheet))
            .collect(),
    }
}

// Apply styles to a single element, returning the specified values
fn specified_values(elem: &ElementData, stylesheet: &StyleSheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    // Go through the rules from lowest to highest specificity
    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector),
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // Check type selector
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // Check id selector
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // Check class selectors
    let elem_classes = elem.classes();
    if selector
        .class
        .iter()
        .any(|class| !elem_classes.contains(&**class))
    {
        return false;
    }

    true
}

pub type MatchedRule<'a> = (Specificity, &'a Rule);

// If `rule` matches `elem`, return a `MatchedRule`. Otherwise return `Node`
fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    // Find the first (highest-specificity) matching selector
    rule.selectors
        .iter()
        .find(|selector| matches(elem, *selector))
        .map(|selector| (selector.specificity(), rule))
}

// Find all CSS Rules that match the given element
fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a StyleSheet) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(elem, rule))
        .collect()
}
