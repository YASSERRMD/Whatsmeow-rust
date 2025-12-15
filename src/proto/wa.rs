//! WhatsApp protobuf message definitions.
//!
//! These are the core protocol buffer definitions for WhatsApp handshake.

use prost::Message;

/// Handshake message for Noise protocol.
#[derive(Clone, PartialEq, Message)]
pub struct HandshakeMessage {
    #[prost(message, optional, tag = "2")]
    pub client_hello: Option<ClientHello>,
    #[prost(message, optional, tag = "3")]
    pub server_hello: Option<ServerHello>,
    #[prost(message, optional, tag = "4")]
    pub client_finish: Option<ClientFinish>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ClientHello {
    #[prost(bytes, optional, tag = "1")]
    pub ephemeral: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ServerHello {
    #[prost(bytes, optional, tag = "1")]
    pub ephemeral: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "2")]
    pub r#static: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "3")]
    pub payload: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ClientFinish {
    #[prost(bytes, optional, tag = "1")]
    pub r#static: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "2")]
    pub payload: Option<Vec<u8>>,
}

/// Client payload sent after handshake.
#[derive(Clone, PartialEq, Message)]
pub struct ClientPayload {
    #[prost(uint64, optional, tag = "1")]
    pub username: Option<u64>,
    #[prost(bool, optional, tag = "3")]
    pub passive: Option<bool>,
    #[prost(message, optional, tag = "5")]
    pub user_agent: Option<UserAgent>,
    #[prost(message, optional, tag = "6")]
    pub web_info: Option<WebInfo>,
    #[prost(string, optional, tag = "7")]
    pub push_name: Option<String>,
    #[prost(int32, optional, tag = "9")]
    pub session_id: Option<i32>,
    #[prost(bool, optional, tag = "10")]
    pub short_connect: Option<bool>,
    #[prost(int32, optional, tag = "12")]
    pub connect_type: Option<i32>,
    #[prost(int32, optional, tag = "13")]
    pub connect_reason: Option<i32>,
    #[prost(int32, repeated, tag = "14")]
    pub shards: Vec<i32>,
    #[prost(message, optional, tag = "15")]
    pub dns_source: Option<DnsSource>,
    #[prost(uint32, optional, tag = "16")]
    pub connect_attempt_count: Option<u32>,
    #[prost(uint32, optional, tag = "18")]
    pub device: Option<u32>,
    #[prost(message, optional, tag = "19")]
    pub device_pairing_data: Option<DevicePairingData>,
    #[prost(int32, optional, tag = "20")]
    pub product: Option<i32>,
    #[prost(bytes, optional, tag = "21")]
    pub fb_cat: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "22")]
    pub fb_user_agent: Option<Vec<u8>>,
    #[prost(bool, optional, tag = "23")]
    pub oc: Option<bool>,
}

#[derive(Clone, PartialEq, Message)]
pub struct UserAgent {
    #[prost(int32, optional, tag = "1")]
    pub platform: Option<i32>,
    #[prost(message, optional, tag = "2")]
    pub app_version: Option<AppVersion>,
    #[prost(int32, optional, tag = "3")]
    pub release_channel: Option<i32>,
    #[prost(string, optional, tag = "4")]
    pub mcc_mnc: Option<String>,
    #[prost(string, optional, tag = "5")]
    pub os_version: Option<String>,
    #[prost(string, optional, tag = "6")]
    pub device: Option<String>,
    #[prost(string, optional, tag = "7")]
    pub lc: Option<String>,
    #[prost(string, optional, tag = "8")]
    pub locale: Option<String>,
    #[prost(string, optional, tag = "15")]
    pub manufacturer: Option<String>,
    #[prost(string, optional, tag = "16")]
    pub os_build_number: Option<String>,
    #[prost(string, optional, tag = "31")]
    pub phone_id: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct AppVersion {
    #[prost(uint32, optional, tag = "1")]
    pub primary: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub secondary: Option<u32>,
    #[prost(uint32, optional, tag = "3")]
    pub tertiary: Option<u32>,
    #[prost(uint32, optional, tag = "4")]
    pub quaternary: Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pub quinary: Option<u32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct WebInfo {
    #[prost(string, optional, tag = "1")]
    pub ref_token: Option<String>,
    #[prost(string, optional, tag = "2")]
    pub version: Option<String>,
    #[prost(message, optional, tag = "3")]
    pub web_sub_platform: Option<WebSubPlatform>,
}

#[derive(Clone, PartialEq, Message)]
pub struct WebSubPlatform {
    #[prost(int32, optional, tag = "1")]
    pub web_sub_platform: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct DnsSource {
    #[prost(int32, optional, tag = "15")]
    pub dns_method: Option<i32>,
    #[prost(bool, optional, tag = "16")]
    pub app_cached: Option<bool>,
}

#[derive(Clone, PartialEq, Message)]
pub struct DevicePairingData {
    #[prost(bytes, optional, tag = "1")]
    pub e_reg_id: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "2")]
    pub e_key_type: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "3")]
    pub e_ident: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "4")]
    pub e_s_key_id: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "5")]
    pub e_s_key_val: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "6")]
    pub e_s_key_sig: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "7")]
    pub build_hash: Option<Vec<u8>>,
    #[prost(bytes, optional, tag = "8")]
    pub device_props: Option<Vec<u8>>,
}

// Platform constants
pub mod platform {
    pub const ANDROID: i32 = 0;
    pub const IOS: i32 = 1;
    pub const WEB: i32 = 14;
    pub const MACOS: i32 = 24;
}

// Connect type constants
pub mod connect_type {
    pub const CELLULAR_UNKNOWN: i32 = 0;
    pub const WIFI: i32 = 1;
}

// Connect reason constants
pub mod connect_reason {
    pub const PUSH: i32 = 0;
    pub const USER_ACTIVATED: i32 = 1;
}

// Web sub-platform constants
pub mod web_sub_platform {
    pub const WEB_BROWSER: i32 = 0;
    pub const DARWIN: i32 = 1;
    pub const WIN32: i32 = 2;
}

// DNS source type constants
pub mod dns_source_type {
    pub const DNS_LOOKUP: i32 = 0;
    pub const FALLBACK: i32 = 1;
}

// Release channel constants
pub mod release_channel {
    pub const RELEASE: i32 = 0;
    pub const BETA: i32 = 1;
}

/// Create a client payload for web connection.
pub fn make_web_client_payload(push_name: Option<&str>) -> ClientPayload {
    ClientPayload {
        username: None,
        passive: Some(false),
        user_agent: Some(UserAgent {
            platform: Some(platform::WEB),
            app_version: Some(AppVersion {
                primary: Some(2),
                secondary: Some(3000),
                tertiary: Some(1012170356),
                quaternary: Some(0),
                quinary: Some(0),
            }),
            release_channel: Some(release_channel::RELEASE),
            mcc_mnc: Some("000000".to_string()),
            os_version: Some("10.15.7".to_string()),
            device: Some("macOS".to_string()),
            lc: Some("US".to_string()),
            locale: Some("en".to_string()),
            manufacturer: Some("Google Chrome".to_string()),
            os_build_number: Some("121.0.6167.184".to_string()),
            phone_id: None,
        }),
        web_info: Some(WebInfo {
            ref_token: None,
            version: Some("2.3000.1012170356".to_string()),
            web_sub_platform: Some(WebSubPlatform {
                web_sub_platform: Some(web_sub_platform::WEB_BROWSER),
            }),
        }),
        push_name: push_name.map(String::from),
        session_id: Some(rand::random()),
        short_connect: Some(true),
        connect_type: Some(connect_type::WIFI),
        connect_reason: Some(connect_reason::USER_ACTIVATED),
        shards: vec![],
        dns_source: Some(DnsSource {
            dns_method: Some(dns_source_type::DNS_LOOKUP),
            app_cached: Some(false),
        }),
        connect_attempt_count: Some(0),
        device: Some(0),
        device_pairing_data: None,
        product: None,
        fb_cat: None,
        fb_user_agent: None,
        oc: Some(false),
    }
}

/// Create device pairing data for registration.
pub fn make_device_pairing_data(
    reg_id: u32,
    identity_key: &[u8; 32],
    signed_prekey_id: u32,
    signed_prekey: &[u8; 32],
    signed_prekey_sig: &[u8; 64],
) -> DevicePairingData {
    // Encode registration ID as big-endian 4 bytes
    let e_reg_id = reg_id.to_be_bytes().to_vec();
    
    // Key type (always 5 for Curve25519)
    let e_key_type = vec![5];
    
    // Identity key with 5 prefix
    let mut e_ident = Vec::with_capacity(33);
    e_ident.push(5);
    e_ident.extend_from_slice(identity_key);
    
    // Signed prekey ID (3 bytes big-endian)
    let e_s_key_id = vec![
        ((signed_prekey_id >> 16) & 0xFF) as u8,
        ((signed_prekey_id >> 8) & 0xFF) as u8,
        (signed_prekey_id & 0xFF) as u8,
    ];
    
    // Signed prekey with 5 prefix
    let mut e_s_key_val = Vec::with_capacity(33);
    e_s_key_val.push(5);
    e_s_key_val.extend_from_slice(signed_prekey);
    
    DevicePairingData {
        e_reg_id: Some(e_reg_id),
        e_key_type: Some(e_key_type),
        e_ident: Some(e_ident),
        e_s_key_id: Some(e_s_key_id),
        e_s_key_val: Some(e_s_key_val),
        e_s_key_sig: Some(signed_prekey_sig.to_vec()),
        build_hash: None,
        device_props: None,
    }
}
