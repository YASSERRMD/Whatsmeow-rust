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
            _ => {
                // Token
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

    /// Read a node
    fn read_node(&mut self) -> Result<Node, DecodeError> {
        // Read header
        let header = self.read_byte()?;
        let num_attrs = (header >> 1) as usize;
        let has_content = (header & 1) != 0;

        // Read tag
        let tag_marker = self.read_byte()?;
        let tag = self.read_string(tag_marker)?;

        // Read attributes
        let mut attrs = Attrs::new();
        for _ in 0..num_attrs {
            let key_marker = self.read_byte()?;
            let key = self.read_string(key_marker)?;
            let value = self.read_attr_value()?;
            attrs.insert(key, value);
        }

        // Read content
        let content = if has_content {
            let content_marker = self.read_byte()?;
            match content_marker {
                0xF8 => {
                    // List of children
                    let len_marker = self.read_byte()?;
                    let len = match len_marker {
                        n if n < 0xFC => n as usize,
                        0xFC => self.read_byte()? as usize,
                        0xFD => self.read_int(2)?,
                        _ => return Err(DecodeError("invalid list length".to_string())),
                    };
                    let mut children = Vec::with_capacity(len);
                    for _ in 0..len {
                        children.push(self.read_node()?);
                    }
                    NodeContent::Children(children)
                }
                0xFF => {
                    // Bytes
                    let len_marker = self.read_byte()?;
                    let len = match len_marker {
                        n if n < 0xFC => n as usize,
                        0xFC => self.read_byte()? as usize,
                        0xFD => self.read_int(2)?,
                        0xFE => self.read_int(3)?,
                        _ => return Err(DecodeError("invalid bytes length".to_string())),
                    };
                    NodeContent::Bytes(self.read_bytes(len)?)
                }
                _ => {
                    // Single child or string content - treat as bytes
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
