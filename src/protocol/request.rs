//! Request/response handling for IQ queries.
//!
//! Handles WhatsApp IQ (Info/Query) protocol messages.

use crate::binary::Node;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::oneshot;

/// Request tracker for IQ messages.
pub struct RequestTracker {
    pending: Arc<RwLock<HashMap<String, oneshot::Sender<Node>>>>,
    counter: Arc<RwLock<u64>>,
}

impl RequestTracker {
    /// Create a new request tracker.
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Generate a new unique request ID.
    pub fn next_id(&self) -> String {
        let mut counter = self.counter.write().unwrap();
        *counter += 1;
        format!("{:X}.{}", rand::random::<u16>(), counter)
    }

    /// Register a pending request and get a receiver for the response.
    pub fn register(&self, id: &str) -> oneshot::Receiver<Node> {
        let (tx, rx) = oneshot::channel();
        self.pending.write().unwrap().insert(id.to_string(), tx);
        rx
    }

    /// Complete a pending request with a response.
    pub fn complete(&self, id: &str, response: Node) -> bool {
        if let Some(tx) = self.pending.write().unwrap().remove(id) {
            tx.send(response).is_ok()
        } else {
            false
        }
    }

    /// Cancel a pending request.
    pub fn cancel(&self, id: &str) {
        self.pending.write().unwrap().remove(id);
    }

    /// Get count of pending requests.
    pub fn pending_count(&self) -> usize {
        self.pending.read().unwrap().len()
    }
}

impl Default for RequestTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Build an IQ get request.
pub fn build_iq_get(id: &str, xmlns: &str, to: Option<&str>) -> Node {
    let mut node = Node::new("iq");
    node.set_attr("id", id);
    node.set_attr("type", "get");
    node.set_attr("xmlns", xmlns);
    if let Some(to) = to {
        node.set_attr("to", to);
    }
    node
}

/// Build an IQ set request.
pub fn build_iq_set(id: &str, xmlns: &str, to: Option<&str>) -> Node {
    let mut node = Node::new("iq");
    node.set_attr("id", id);
    node.set_attr("type", "set");
    node.set_attr("xmlns", xmlns);
    if let Some(to) = to {
        node.set_attr("to", to);
    }
    node
}

/// Build an IQ result response.
pub fn build_iq_result(id: &str, to: Option<&str>) -> Node {
    let mut node = Node::new("iq");
    node.set_attr("id", id);
    node.set_attr("type", "result");
    if let Some(to) = to {
        node.set_attr("to", to);
    }
    node
}

/// Check if a node is an IQ result.
pub fn is_iq_result(node: &Node) -> bool {
    node.tag == "iq" && node.get_attr_str("type") == Some("result")
}

/// Check if a node is an IQ error.
pub fn is_iq_error(node: &Node) -> bool {
    node.tag == "iq" && node.get_attr_str("type") == Some("error")
}

/// Extract error message from IQ error node.
pub fn get_iq_error(node: &Node) -> Option<String> {
    if !is_iq_error(node) {
        return None;
    }
    
    node.get_child_by_tag("error")
        .and_then(|e| e.get_attr_str("text"))
        .map(String::from)
        .or_else(|| {
            node.get_child_by_tag("error")
                .map(|e| e.tag.clone())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_tracker() {
        let tracker = RequestTracker::new();
        
        let id = tracker.next_id();
        let rx = tracker.register(&id);
        
        assert_eq!(tracker.pending_count(), 1);
        
        let response = Node::new("result");
        assert!(tracker.complete(&id, response));
        
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn test_build_iq_get() {
        let node = build_iq_get("123", "w:profile:picture", Some("user@server"));
        
        assert_eq!(node.tag, "iq");
        assert_eq!(node.get_attr_str("type"), Some("get"));
        assert_eq!(node.get_attr_str("xmlns"), Some("w:profile:picture"));
    }

    #[test]
    fn test_is_iq_result() {
        let mut result = Node::new("iq");
        result.set_attr("type", "result");
        assert!(is_iq_result(&result));
        
        let mut error = Node::new("iq");
        error.set_attr("type", "error");
        assert!(!is_iq_result(&error));
        assert!(is_iq_error(&error));
    }
}
