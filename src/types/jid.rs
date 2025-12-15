//! WhatsApp JID (Jabber ID) types.
//!
//! JIDs are used to identify users, groups, and other entities in WhatsApp.

use std::fmt;
use std::str::FromStr;

/// Known JID servers on WhatsApp
pub mod servers {
    pub const DEFAULT_USER: &str = "s.whatsapp.net";
    pub const GROUP: &str = "g.us";
    pub const LEGACY_USER: &str = "c.us";
    pub const BROADCAST: &str = "broadcast";
    pub const HIDDEN_USER: &str = "lid";
    pub const MESSENGER: &str = "msgr";
    pub const INTEROP: &str = "interop";
    pub const NEWSLETTER: &str = "newsletter";
    pub const HOSTED: &str = "hosted";
    pub const HOSTED_LID: &str = "hosted.lid";
    pub const BOT: &str = "bot";
}

/// Domain type constants
pub const WHATSAPP_DOMAIN: u8 = 0;
pub const LID_DOMAIN: u8 = 1;
pub const HOSTED_DOMAIN: u8 = 128;
pub const HOSTED_LID_DOMAIN: u8 = 129;

/// MessageID is the internal ID of a WhatsApp message.
pub type MessageID = String;

/// MessageServerID is the server ID of a WhatsApp newsletter message.
pub type MessageServerID = i32;

/// JID represents a WhatsApp user ID.
///
/// There are two types of JIDs:
/// - Regular JID pairs (user and server)
/// - AD-JIDs (user, agent and device) for specific devices
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct JID {
    pub user: String,
    pub raw_agent: u8,
    pub device: u16,
    pub integrator: u16,
    pub server: String,
}

impl JID {
    /// Creates a new regular JID.
    pub fn new(user: impl Into<String>, server: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            server: server.into(),
            ..Default::default()
        }
    }

    /// Creates a new AD JID with agent and device.
    pub fn new_ad(user: impl Into<String>, agent: u8, device: u8) -> Self {
        let user = user.into();
        let (server, raw_agent) = match agent {
            LID_DOMAIN => (servers::HIDDEN_USER.to_string(), 0),
            HOSTED_DOMAIN => (servers::HOSTED.to_string(), 0),
            HOSTED_LID_DOMAIN => (servers::HOSTED_LID.to_string(), 0),
            _ => (servers::DEFAULT_USER.to_string(), agent),
        };

        Self {
            user,
            raw_agent,
            device: device as u16,
            server,
            integrator: 0,
        }
    }

    /// Returns the actual agent/domain type.
    pub fn actual_agent(&self) -> u8 {
        match self.server.as_str() {
            servers::DEFAULT_USER => WHATSAPP_DOMAIN,
            servers::HIDDEN_USER => LID_DOMAIN,
            servers::HOSTED => HOSTED_DOMAIN,
            servers::HOSTED_LID => HOSTED_LID_DOMAIN,
            _ => self.raw_agent,
        }
    }

    /// Returns the user as an integer (only safe for normal users).
    pub fn user_int(&self) -> u64 {
        self.user.parse().unwrap_or(0)
    }

    /// Returns a version of the JID without agent and device.
    pub fn to_non_ad(&self) -> Self {
        Self {
            user: self.user.clone(),
            server: self.server.clone(),
            integrator: self.integrator,
            ..Default::default()
        }
    }

    /// Returns true if this is a broadcast list (not status broadcast).
    pub fn is_broadcast_list(&self) -> bool {
        self.server == servers::BROADCAST && self.user != "status"
    }

    /// Returns true if this JID represents a bot.
    pub fn is_bot(&self) -> bool {
        if self.server == servers::BOT {
            return true;
        }
        if self.server == servers::DEFAULT_USER && self.device == 0 {
            // Check bot user patterns
            if let Some(first_chars) = self.user.get(0..7) {
                if first_chars == "1313555" && self.user.len() == 11 {
                    return true;
                }
            }
            if let Some(first_chars) = self.user.get(0..9) {
                if first_chars == "131655500" && self.user.len() == 11 {
                    return true;
                }
            }
        }
        false
    }

    /// Returns true if the JID is empty (no server).
    pub fn is_empty(&self) -> bool {
        self.server.is_empty()
    }

    /// Returns the AD string representation.
    pub fn ad_string(&self) -> String {
        format!("{}.{}:{}@{}", self.user, self.raw_agent, self.device, self.server)
    }

    /// Returns the signal address user string.
    pub fn signal_address_user(&self) -> String {
        let agent = self.actual_agent();
        if agent != 0 {
            format!("{}_{}", self.user, agent)
        } else {
            self.user.clone()
        }
    }
}

impl fmt::Display for JID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw_agent > 0 {
            write!(f, "{}.{}:{}@{}", self.user, self.raw_agent, self.device, self.server)
        } else if self.device > 0 {
            write!(f, "{}:{}@{}", self.user, self.device, self.server)
        } else if !self.user.is_empty() {
            write!(f, "{}@{}", self.user, self.server)
        } else {
            write!(f, "{}", self.server)
        }
    }
}

/// Error type for JID parsing
#[derive(Debug, Clone, PartialEq)]
pub struct ParseJIDError(pub String);

impl fmt::Display for ParseJIDError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse JID: {}", self.0)
    }
}

impl std::error::Error for ParseJIDError {}

impl FromStr for JID {
    type Err = ParseJIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        
        if parts.len() == 1 {
            return Ok(JID::new("", parts[0]));
        }

        let user_str = parts[0].to_string();
        let server = parts[1].to_string();

        let mut jid = JID {
            user: user_str.clone(),
            server,
            ..Default::default()
        };

        // Check for AD JID format (user.agent:device@server)
        if user_str.contains('.') {
            let user_parts: Vec<&str> = user_str.split('.').collect();
            if user_parts.len() != 2 {
                return Err(ParseJIDError("unexpected number of dots in JID".to_string()));
            }
            jid.user = user_parts[0].to_string();
            let ad = user_parts[1];
            
            let ad_parts: Vec<&str> = ad.split(':').collect();
            if ad_parts.len() > 2 {
                return Err(ParseJIDError("unexpected number of colons in JID".to_string()));
            }
            
            jid.raw_agent = ad_parts[0].parse()
                .map_err(|_| ParseJIDError("failed to parse agent from JID".to_string()))?;
            
            if ad_parts.len() == 2 {
                jid.device = ad_parts[1].parse()
                    .map_err(|_| ParseJIDError("failed to parse device from JID".to_string()))?;
            }
        } else if user_str.contains(':') {
            let user_parts: Vec<&str> = user_str.split(':').collect();
            if user_parts.len() != 2 {
                return Err(ParseJIDError("unexpected number of colons in JID".to_string()));
            }
            jid.user = user_parts[0].to_string();
            jid.device = user_parts[1].parse()
                .map_err(|_| ParseJIDError("failed to parse device from JID".to_string()))?;
        }

        Ok(jid)
    }
}

// Common JIDs
lazy_static::lazy_static! {
    pub static ref EMPTY_JID: JID = JID::default();
    pub static ref GROUP_SERVER_JID: JID = JID::new("", servers::GROUP);
    pub static ref SERVER_JID: JID = JID::new("", servers::DEFAULT_USER);
    pub static ref BROADCAST_SERVER_JID: JID = JID::new("", servers::BROADCAST);
    pub static ref STATUS_BROADCAST_JID: JID = JID::new("status", servers::BROADCAST);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_jid() {
        let jid: JID = "1234567890@s.whatsapp.net".parse().unwrap();
        assert_eq!(jid.user, "1234567890");
        assert_eq!(jid.server, servers::DEFAULT_USER);
        assert_eq!(jid.device, 0);
        assert_eq!(jid.raw_agent, 0);
    }

    #[test]
    fn test_parse_device_jid() {
        let jid: JID = "1234567890:2@s.whatsapp.net".parse().unwrap();
        assert_eq!(jid.user, "1234567890");
        assert_eq!(jid.server, servers::DEFAULT_USER);
        assert_eq!(jid.device, 2);
    }

    #[test]
    fn test_parse_ad_jid() {
        let jid: JID = "1234567890.0:1@s.whatsapp.net".parse().unwrap();
        assert_eq!(jid.user, "1234567890");
        assert_eq!(jid.raw_agent, 0);
        assert_eq!(jid.device, 1);
    }

    #[test]
    fn test_jid_to_string() {
        let jid = JID::new("1234567890", servers::DEFAULT_USER);
        assert_eq!(jid.to_string(), "1234567890@s.whatsapp.net");

        let jid = JID {
            user: "1234567890".to_string(),
            device: 2,
            server: servers::DEFAULT_USER.to_string(),
            ..Default::default()
        };
        assert_eq!(jid.to_string(), "1234567890:2@s.whatsapp.net");
    }

    #[test]
    fn test_group_jid() {
        let jid: JID = "123456789-1234567890@g.us".parse().unwrap();
        assert_eq!(jid.user, "123456789-1234567890");
        assert_eq!(jid.server, servers::GROUP);
    }
}
