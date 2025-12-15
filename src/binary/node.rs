//! Binary XML Node type for WhatsApp protocol.
//!
//! WhatsApp uses a custom binary XML format for message encoding.
//! This module provides the Node type and serialization.

use std::collections::HashMap;
use crate::types::JID;

/// Attributes of an XML node
pub type Attrs = HashMap<String, AttrValue>;

/// Possible values for node attributes
#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    None,
    String(String),
    Bytes(Vec<u8>),
    Int(i64),
    Bool(bool),
    JID(JID),
}

impl From<&str> for AttrValue {
    fn from(s: &str) -> Self {
        AttrValue::String(s.to_string())
    }
}

impl From<String> for AttrValue {
    fn from(s: String) -> Self {
        AttrValue::String(s)
    }
}

impl From<i64> for AttrValue {
    fn from(n: i64) -> Self {
        AttrValue::Int(n)
    }
}

impl From<bool> for AttrValue {
    fn from(b: bool) -> Self {
        AttrValue::Bool(b)
    }
}

impl From<JID> for AttrValue {
    fn from(jid: JID) -> Self {
        AttrValue::JID(jid)
    }
}

impl AttrValue {
    /// Get as string if possible
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AttrValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as i64 if possible
    pub fn as_int(&self) -> Option<i64> {
        match self {
            AttrValue::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as JID if possible
    pub fn as_jid(&self) -> Option<&JID> {
        match self {
            AttrValue::JID(jid) => Some(jid),
            _ => None,
        }
    }
}

/// Node represents a binary XML element in WhatsApp protocol.
#[derive(Debug, Clone, Default)]
pub struct Node {
    /// The tag name of the element
    pub tag: String,
    /// The attributes of the element
    pub attrs: Attrs,
    /// The content inside the element (nil, children, or bytes)
    pub content: NodeContent,
}

/// Content of a node
#[derive(Debug, Clone, Default)]
pub enum NodeContent {
    #[default]
    None,
    /// Child nodes
    Children(Vec<Node>),
    /// Binary data
    Bytes(Vec<u8>),
}

impl Node {
    /// Create a new node with the given tag
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attrs: Attrs::new(),
            content: NodeContent::None,
        }
    }

    /// Create a new node with tag and attributes
    pub fn with_attrs(tag: impl Into<String>, attrs: Attrs) -> Self {
        Self {
            tag: tag.into(),
            attrs,
            content: NodeContent::None,
        }
    }

    /// Set an attribute on this node
    pub fn set_attr(&mut self, key: impl Into<String>, value: impl Into<AttrValue>) {
        self.attrs.insert(key.into(), value.into());
    }

    /// Get an attribute value
    pub fn get_attr(&self, key: &str) -> Option<&AttrValue> {
        self.attrs.get(key)
    }

    /// Get an attribute as string
    pub fn get_attr_str(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).and_then(|v| v.as_str())
    }

    /// Get an attribute as int
    pub fn get_attr_int(&self, key: &str) -> Option<i64> {
        self.attrs.get(key).and_then(|v| v.as_int())
    }

    /// Get an attribute as JID
    pub fn get_attr_jid(&self, key: &str) -> Option<&JID> {
        self.attrs.get(key).and_then(|v| v.as_jid())
    }

    /// Set the content to child nodes
    pub fn set_children(&mut self, children: Vec<Node>) {
        self.content = NodeContent::Children(children);
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Node) {
        match &mut self.content {
            NodeContent::Children(children) => children.push(child),
            _ => self.content = NodeContent::Children(vec![child]),
        }
    }

    /// Set the content to bytes
    pub fn set_bytes(&mut self, bytes: Vec<u8>) {
        self.content = NodeContent::Bytes(bytes);
    }

    /// Get children if content is children
    pub fn get_children(&self) -> Option<&[Node]> {
        match &self.content {
            NodeContent::Children(children) => Some(children),
            _ => None,
        }
    }

    /// Get children by tag
    pub fn get_children_by_tag(&self, tag: &str) -> Vec<&Node> {
        match &self.content {
            NodeContent::Children(children) => {
                children.iter().filter(|n| n.tag == tag).collect()
            }
            _ => Vec::new(),
        }
    }

    /// Get first child with the given tag
    pub fn get_child_by_tag(&self, tag: &str) -> Option<&Node> {
        self.get_children_by_tag(tag).into_iter().next()
    }

    /// Get an optional child by walking through nested tags
    pub fn get_optional_child_by_tag(&self, tags: &[&str]) -> Option<&Node> {
        let mut current = self;
        for tag in tags {
            current = current.get_child_by_tag(tag)?;
        }
        Some(current)
    }

    /// Get bytes content if present
    pub fn get_bytes(&self) -> Option<&[u8]> {
        match &self.content {
            NodeContent::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let mut node = Node::new("message");
        node.set_attr("id", "123");
        node.set_attr("type", "text");

        assert_eq!(node.tag, "message");
        assert_eq!(node.get_attr_str("id"), Some("123"));
        assert_eq!(node.get_attr_str("type"), Some("text"));
    }

    #[test]
    fn test_node_children() {
        let mut parent = Node::new("iq");
        let child1 = Node::new("query");
        let child2 = Node::new("result");

        parent.add_child(child1);
        parent.add_child(child2);

        let children = parent.get_children().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].tag, "query");
        assert_eq!(children[1].tag, "result");
    }

    #[test]
    fn test_node_bytes() {
        let mut node = Node::new("media");
        node.set_bytes(vec![1, 2, 3, 4]);

        assert_eq!(node.get_bytes(), Some(&[1, 2, 3, 4][..]));
    }
}
