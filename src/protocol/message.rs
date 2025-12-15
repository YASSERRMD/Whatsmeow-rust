//! Message handling for WhatsApp protocol.
//!
//! Provides message building, sending, and receiving functionality.

use crate::types::{JID, MessageContent, MessageInfo};
use crate::binary::{Node, AttrValue};
use chrono::Utc;
use rand::Rng;

/// Generate a unique message ID.
pub fn generate_message_id() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 8] = rng.gen();
    let id: u64 = u64::from_be_bytes(bytes);
    format!("{:X}", id)
}

/// Build a text message node.
pub fn build_text_message(to: &JID, text: &str, message_id: Option<&str>) -> Node {
    let id = message_id.map(String::from).unwrap_or_else(generate_message_id);
    
    let mut node = Node::new("message");
    node.set_attr("id", id);
    node.set_attr("type", "text");
    node.set_attr("to", to.to_string());
    
    let mut body = Node::new("body");
    body.set_bytes(text.as_bytes().to_vec());
    node.add_child(body);
    
    node
}

/// Build a media message node.
pub fn build_media_message(
    to: &JID,
    media_type: &str,
    url: &str,
    mimetype: &str,
    caption: Option<&str>,
) -> Node {
    let id = generate_message_id();
    
    let mut node = Node::new("message");
    node.set_attr("id", id);
    node.set_attr("type", "media");
    node.set_attr("to", to.to_string());
    node.set_attr("mediatype", media_type);
    
    let mut media = Node::new("media");
    media.set_attr("type", media_type);
    media.set_attr("url", url);
    media.set_attr("mimetype", mimetype);
    if let Some(caption) = caption {
        let mut caption_node = Node::new("caption");
        caption_node.set_bytes(caption.as_bytes().to_vec());
        media.add_child(caption_node);
    }
    node.add_child(media);
    
    node
}

/// Build a receipt node.
pub fn build_receipt(to: &JID, message_ids: &[String], receipt_type: &str) -> Node {
    let mut node = Node::new("receipt");
    node.set_attr("to", to.to_string());
    node.set_attr("type", receipt_type);
    
    for id in message_ids {
        let mut item = Node::new("item");
        item.set_attr("id", id.clone());
        node.add_child(item);
    }
    
    node
}

/// Build a read receipt node.
pub fn build_read_receipt(to: &JID, message_ids: &[String]) -> Node {
    build_receipt(to, message_ids, "read")
}

/// Build a presence node.
pub fn build_presence(available: bool) -> Node {
    let mut node = Node::new("presence");
    node.set_attr("type", if available { "available" } else { "unavailable" });
    node
}

/// Build a typing indicator node.
pub fn build_chat_state(to: &JID, composing: bool) -> Node {
    let mut node = Node::new("chatstate");
    node.set_attr("to", to.to_string());
    
    let state = Node::new(if composing { "composing" } else { "paused" });
    node.add_child(state);
    
    node
}

/// Parse a message node into MessageInfo and MessageContent.
pub fn parse_message(node: &Node) -> Option<(MessageInfo, MessageContent)> {
    if node.tag != "message" {
        return None;
    }
    
    let id = node.get_attr_str("id")?.to_string();
    let from_str = node.get_attr_str("from")?;
    let from: JID = from_str.parse().ok()?;
    let msg_type = node.get_attr_str("type").unwrap_or("text");
    
    let is_group = from.server == crate::types::servers::GROUP;
    let sender = if is_group {
        node.get_attr_str("participant")
            .and_then(|s| s.parse().ok())
            .unwrap_or(from.clone())
    } else {
        from.clone()
    };
    
    let info = MessageInfo {
        id,
        sender,
        chat: from,
        is_from_me: false, // Will be determined by comparing to own JID
        is_group,
        timestamp: Utc::now().timestamp(),
        push_name: node.get_attr_str("notify").map(String::from),
    };
    
    let content = match msg_type {
        "text" => {
            let body = node.get_child_by_tag("body")
                .and_then(|b| b.get_bytes())
                .map(|b| String::from_utf8_lossy(b).to_string())
                .unwrap_or_default();
            MessageContent::Text(body)
        }
        "media" => {
            parse_media_content(node).unwrap_or(MessageContent::Unknown)
        }
        _ => MessageContent::Unknown,
    };
    
    Some((info, content))
}

/// Parse media content from a message node.
fn parse_media_content(node: &Node) -> Option<MessageContent> {
    let media = node.get_child_by_tag("media")?;
    let media_type = media.get_attr_str("type")?;
    let url = media.get_attr_str("url")?.to_string();
    let mimetype = media.get_attr_str("mimetype").unwrap_or("application/octet-stream").to_string();
    let caption = media.get_child_by_tag("caption")
        .and_then(|c| c.get_bytes())
        .map(|b| String::from_utf8_lossy(b).to_string());
    
    Some(match media_type {
        "image" => MessageContent::Image { url, caption, mimetype },
        "video" => MessageContent::Video { url, caption, mimetype },
        "audio" => MessageContent::Audio { url, mimetype, ptt: false },
        "document" => MessageContent::Document {
            url,
            filename: media.get_attr_str("filename").unwrap_or("file").to_string(),
            mimetype,
        },
        "sticker" => MessageContent::Sticker { url },
        _ => MessageContent::Unknown,
    })
}

/// Parse a receipt node.
pub fn parse_receipt(node: &Node) -> Option<(JID, Vec<String>, String)> {
    if node.tag != "receipt" {
        return None;
    }
    
    let from: JID = node.get_attr_str("from")?.parse().ok()?;
    let receipt_type = node.get_attr_str("type").unwrap_or("delivery").to_string();
    
    let message_ids: Vec<String> = node.get_children()
        .map(|children| {
            children.iter()
                .filter(|n| n.tag == "item")
                .filter_map(|n| n.get_attr_str("id").map(String::from))
                .collect()
        })
        .unwrap_or_default();
    
    Some((from, message_ids, receipt_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_message_id() {
        let id1 = generate_message_id();
        let id2 = generate_message_id();
        
        assert!(!id1.is_empty());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_build_text_message() {
        let to = JID::new("123456789", "s.whatsapp.net");
        let node = build_text_message(&to, "Hello, World!", None);
        
        assert_eq!(node.tag, "message");
        assert_eq!(node.get_attr_str("type"), Some("text"));
        assert!(node.get_attr_str("id").is_some());
    }

    #[test]
    fn test_build_presence() {
        let available = build_presence(true);
        assert_eq!(available.tag, "presence");
        assert_eq!(available.get_attr_str("type"), Some("available"));
        
        let unavailable = build_presence(false);
        assert_eq!(unavailable.get_attr_str("type"), Some("unavailable"));
    }

    #[test]
    fn test_build_chat_state() {
        let to = JID::new("123456789", "s.whatsapp.net");
        let composing = build_chat_state(&to, true);
        
        assert_eq!(composing.tag, "chatstate");
        let children = composing.get_children().unwrap();
        assert_eq!(children[0].tag, "composing");
    }
}
