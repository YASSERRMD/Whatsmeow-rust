//! Binary encoder for WhatsApp protocol.
//!
//! Encodes Node structures into WhatsApp's binary XML format.

use super::node::{Node, NodeContent, AttrValue};
use super::token::get_token_index;

/// Binary encoder for WhatsApp XML nodes
pub struct Encoder {
    data: Vec<u8>,
}

impl Encoder {
    /// Create a new encoder
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Encode a node and return the binary data
    pub fn encode(node: &Node) -> Vec<u8> {
        let mut encoder = Self::new();
        encoder.write_node(node);
        encoder.data
    }

    /// Write a byte
    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    /// Write multiple bytes
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Write an integer with variable byte encoding
    fn write_int(&mut self, n: usize) {
        if n < 256 {
            self.write_byte(n as u8);
        } else if n < 65536 {
            self.write_byte(((n >> 8) & 0xFF) as u8);
            self.write_byte((n & 0xFF) as u8);
        } else {
            self.write_byte(((n >> 16) & 0xFF) as u8);
            self.write_byte(((n >> 8) & 0xFF) as u8);
            self.write_byte((n & 0xFF) as u8);
        }
    }

    /// Write a string (possibly as token)
    fn write_string(&mut self, s: &str) {
        if s.is_empty() {
            self.write_byte(0xFC); // Empty string marker
            return;
        }

        // Try to use a token
        if let Some(token) = get_token_index(s) {
            self.write_byte(token);
            return;
        }

        // Write as raw string
        let bytes = s.as_bytes();
        if bytes.len() < 256 {
            self.write_byte(0xFC); // Short string marker
            self.write_byte(bytes.len() as u8);
        } else if bytes.len() < 65536 {
            self.write_byte(0xFD); // Medium string marker
            self.write_byte(((bytes.len() >> 8) & 0xFF) as u8);
            self.write_byte((bytes.len() & 0xFF) as u8);
        } else {
            self.write_byte(0xFE); // Long string marker
            self.write_byte(((bytes.len() >> 16) & 0xFF) as u8);
            self.write_byte(((bytes.len() >> 8) & 0xFF) as u8);
            self.write_byte((bytes.len() & 0xFF) as u8);
        }
        self.write_bytes(bytes);
    }

    /// Write an attribute value
    fn write_attr_value(&mut self, value: &AttrValue) {
        match value {
            AttrValue::None => self.write_byte(0x00),
            AttrValue::String(s) => self.write_string(s),
            AttrValue::Bytes(b) => {
                self.write_byte(0xFF); // Bytes marker
                self.write_int(b.len());
                self.write_bytes(b);
            }
            AttrValue::Int(n) => {
                // Write as string representation
                self.write_string(&n.to_string());
            }
            AttrValue::Bool(b) => {
                self.write_string(if *b { "true" } else { "false" });
            }
            AttrValue::JID(jid) => {
                self.write_jid(jid);
            }
        }
    }

    /// Write a JID
    fn write_jid(&mut self, jid: &crate::types::JID) {
        if jid.raw_agent > 0 || jid.device > 0 {
            // AD JID
            self.write_byte(0xFA); // AD JID marker
            self.write_byte(jid.raw_agent);
            self.write_byte(jid.device as u8);
            self.write_string(&jid.user);
        } else {
            // Regular JID - write as user@server
            self.write_byte(0xF9); // JID marker
            self.write_string(&jid.user);
            self.write_string(&jid.server);
        }
    }

    /// Write a node
    fn write_node(&mut self, node: &Node) {
        // Determine number of attributes
        let num_attrs = node.attrs.len();
        
        // Determine if there's content
        let has_content = !matches!(node.content, NodeContent::None);

        // Write node header
        // Format: 1 byte for number of attrs + has_content flag
        let header = ((num_attrs << 1) | (if has_content { 1 } else { 0 })) as u8;
        self.write_byte(header);

        // Write tag
        self.write_string(&node.tag);

        // Write attributes
        for (key, value) in &node.attrs {
            self.write_string(key);
            self.write_attr_value(value);
        }

        // Write content
        match &node.content {
            NodeContent::None => {}
            NodeContent::Children(children) => {
                // Write list header
                self.write_byte(0xF8); // List marker
                self.write_int(children.len());
                for child in children {
                    self.write_node(child);
                }
            }
            NodeContent::Bytes(bytes) => {
                self.write_byte(0xFF); // Bytes marker
                self.write_int(bytes.len());
                self.write_bytes(bytes);
            }
        }
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode a node to binary format
pub fn encode(node: &Node) -> Vec<u8> {
    Encoder::encode(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple_node() {
        let mut node = Node::new("message");
        node.set_attr("id", "123");
        
        let encoded = encode(&node);
        assert!(!encoded.is_empty());
    }
}
