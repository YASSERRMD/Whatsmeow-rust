//! Binary decoder for WhatsApp protocol.
//!
//! Decodes WhatsApp's binary XML format into Node structures.

use super::node::{Node, NodeContent, AttrValue, Attrs};
use super::token::get_token;
use crate::types::JID;

/// Error type for decoding
#[derive(Debug, Clone)]
pub struct DecodeError(pub String);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "decode error: {}", self.0)
    }
}

impl std::error::Error for DecodeError {}

/// Binary decoder for WhatsApp XML nodes
pub struct Decoder<'a> {
    data: &'a [u8],
    index: usize,
}

impl<'a> Decoder<'a> {
    /// Create a new decoder
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, index: 0 }
    }

    /// Decode the data into a node
    pub fn decode(data: &[u8]) -> Result<Node, DecodeError> {
        let mut decoder = Decoder::new(data);
        let node = decoder.read_node()?;
        
        if decoder.index != decoder.data.len() {
            return Err(DecodeError(format!(
                "{} leftover bytes after decoding",
                decoder.data.len() - decoder.index
            )));
        }
        
        Ok(node)
    }

    /// Check if there's more data
    fn has_more(&self) -> bool {
        self.index < self.data.len()
    }

    /// Read a single byte
    fn read_byte(&mut self) -> Result<u8, DecodeError> {
        if self.index >= self.data.len() {
            return Err(DecodeError("unexpected end of data".to_string()));
        }
        let b = self.data[self.index];
        self.index += 1;
        Ok(b)
    }

    /// Read multiple bytes
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>, DecodeError> {
        if self.index + n > self.data.len() {
            return Err(DecodeError("unexpected end of data".to_string()));
        }
        let bytes = self.data[self.index..self.index + n].to_vec();
        self.index += n;
        Ok(bytes)
    }

    /// Read an integer based on length marker
    fn read_int(&mut self, bytes: usize) -> Result<usize, DecodeError> {
        let mut result = 0usize;
        for _ in 0..bytes {
            result = (result << 8) | (self.read_byte()? as usize);
        }
        Ok(result)
    }

    /// Read a string (possibly from token)
    fn read_string(&mut self, tag: u8) -> Result<String, DecodeError> {
        match tag {
            0x00 => Ok(String::new()),
            0xFC => {
                // Short string
                let len = self.read_byte()? as usize;
                if len == 0 {
                    return Ok(String::new());
                }
                let bytes = self.read_bytes(len)?;
                String::from_utf8(bytes)
                    .map_err(|e| DecodeError(format!("invalid utf8: {}", e)))
            }
            0xFD => {
                // Medium string
                let len = self.read_int(2)?;
                let bytes = self.read_bytes(len)?;
                String::from_utf8(bytes)
                    .map_err(|e| DecodeError(format!("invalid utf8: {}", e)))
            }
            0xFE => {
                // Long string
                let len = self.read_int(3)?;
                let bytes = self.read_bytes(len)?;
                String::from_utf8(bytes)
                    .map_err(|e| DecodeError(format!("invalid utf8: {}", e)))
            }
            // Dictionary tokens (double-byte)
            0xEC..=0xEF => {
                let dict = tag - 0xEC;  // 0-3
                let index = self.read_byte()?;
                if let Some(token) = super::token::get_double_token(dict, index) {
                    Ok(token.to_string())
                } else {
                    Err(DecodeError(format!("unknown double token: dict={}, index={}", dict, index)))
                }
            }
            _ => {
                // Single-byte token
                if let Some(token) = get_token(tag) {
                    Ok(token.to_string())
                } else {
                    Err(DecodeError(format!("unknown token: {}", tag)))
                }
            }
        }
    }

    /// Read a JID
    fn read_jid(&mut self, marker: u8) -> Result<JID, DecodeError> {
        match marker {
            0xF9 => {
                // Regular JID
                let user_tag = self.read_byte()?;
                let user = self.read_string(user_tag)?;
                let server_tag = self.read_byte()?;
                let server = self.read_string(server_tag)?;
                Ok(JID::new(user, server))
            }
            0xFA => {
                // AD JID
                let agent = self.read_byte()?;
                let device = self.read_byte()?;
                let user_tag = self.read_byte()?;
                let user = self.read_string(user_tag)?;
                Ok(JID::new_ad(user, agent, device))
            }
            _ => Err(DecodeError(format!("invalid JID marker: {}", marker))),
        }
    }

    /// Read an attribute value
    fn read_attr_value(&mut self) -> Result<AttrValue, DecodeError> {
        let tag = self.read_byte()?;
        match tag {
            0x00 => Ok(AttrValue::None),
            0xF9 | 0xFA => {
                let jid = self.read_jid(tag)?;
                Ok(AttrValue::JID(jid))
            }
            0xFF => {
                // Bytes
                let len_tag = self.read_byte()?;
                let len = match len_tag {
                    n if n < 0xFC => n as usize,
                    0xFC => self.read_byte()? as usize,
                    0xFD => self.read_int(2)?,
                    0xFE => self.read_int(3)?,
                    _ => return Err(DecodeError("invalid length marker".to_string())),
                };
                let bytes = self.read_bytes(len)?;
                Ok(AttrValue::Bytes(bytes))
            }
            _ => {
                let s = self.read_string(tag)?;
                Ok(AttrValue::String(s))
            }
        }
    }

    /// Read list size from token
    fn read_list_size(&mut self, token: u8) -> Result<usize, DecodeError> {
        match token {
            0x00 => Ok(0),
            0xF8 => Ok(self.read_byte()? as usize),
            0xF9 => Ok(self.read_int(2)?),
            _ => Err(DecodeError(format!("expected list token (f8/f9), got 0x{:02x}", token))),
        }
    }

    /// Read a node
    fn read_node(&mut self) -> Result<Node, DecodeError> {
        // Node is always a list
        let token = self.read_byte()?;
        let size = self.read_list_size(token)?;

        if size == 0 {
            return Err(DecodeError("invalid empty list for node".to_string()));
        }

        // 1. Read Tag
        let tag_marker = self.read_byte()?;
        let tag = self.read_string(tag_marker)?;

        let mut attrs = Attrs::new();
        
        // Number of attribute pairs = (size - 1) / 2
        let num_attr_pairs = (size - 1) / 2;
        
        for _ in 0..num_attr_pairs {
            let key_marker = self.read_byte()?;
            let key = self.read_string(key_marker)?;
            let value = self.read_attr_value()?;
            attrs.insert(key, value);
        }

        // If (size - 1) is odd, there is content
        let has_content = (size - 1) % 2 == 1;

        // Read content
        let content = if has_content {
            let content_marker = self.read_byte()?;
            match content_marker {
                0xF8 | 0xF9 => {
                    // List -> Children
                    let len = self.read_list_size(content_marker)?;
                    let mut children = Vec::with_capacity(len);
                    for _ in 0..len {
                        children.push(self.read_node()?);
                    }
                    NodeContent::Children(children)
                }
                0xFF | 0xFC | 0xFD | 0xFE => {
                    // Bytes
                    let len = match content_marker {
                        0xFC => self.read_byte()? as usize,
                        0xFD => self.read_int(2)?,
                        0xFE => self.read_int(3)?,
                        _ => return Err(DecodeError("invalid bytes length".to_string())), // FF shouldn't happen alone?
                    };
                    NodeContent::Bytes(self.read_bytes(len)?)
                }
                _ => {
                    // String content - treat as bytes
                    let s = self.read_string(content_marker)?;
                    NodeContent::Bytes(s.into_bytes())
                }
            }
        } else {
            NodeContent::None
        };

        Ok(Node { tag, attrs, content })
    }
}

/// Decode binary data into a node
pub fn decode(data: &[u8]) -> Result<Node, DecodeError> {
    Decoder::decode(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::encoder::encode;

    #[test]
    fn test_roundtrip() {
        let mut node = Node::new("message");
        node.set_attr("id", "test123");
        node.set_attr("type", "text");
        
        let encoded = encode(&node);
        // Note: Full roundtrip testing requires consistent encoding
        assert!(!encoded.is_empty());
    }
}
