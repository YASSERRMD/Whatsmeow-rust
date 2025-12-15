//! Token dictionary for WhatsApp binary XML encoding.
//!
//! WhatsApp uses a dictionary of common strings to compress binary XML messages.
//! Instead of sending the full string, a token byte is sent that maps to the string.

/// Single-byte tokens (0-235)
pub static SINGLE_BYTE_TOKENS: &[&str] = &[
    "",                       // 0
    "xmlstreamstart",         // 1
    "xmlstreamend",           // 2
    "s.whatsapp.net",         // 3
    "type",                   // 4
    "participant",            // 5
    "from",                   // 6
    "receipt",                // 7
    "id",                     // 8
    "notification",           // 9
    "disappearing_mode",      // 10
    "status",                 // 11
    "jid",                    // 12
    "broadcast",              // 13
    "user",                   // 14
    "devices",                // 15
    "device_hash",            // 16
    "to",                     // 17
    "offline",                // 18
    "message",                // 19
    "result",                 // 20
    "class",                  // 21
    "xmlns",                  // 22
    "duration",               // 23
    "notify",                 // 24
    "iq",                     // 25
    "t",                      // 26
    "ack",                    // 27
    "g.us",                   // 28
    "enc",                    // 29
    "urn:xmpp:whatsapp:push", // 30
    "presence",               // 31
    "config_value",           // 32
    "picture",                // 33
    "verified_name",          // 34
    "config_code",            // 35
    "key-index-list",         // 36
    "contact",                // 37
    "mediatype",              // 38
    "routing_info",           // 39
    "edge_routing",           // 40
    "get",                    // 41
    "read",                   // 42
    "urn:xmpp:ping",          // 43
    "fallback_hostname",      // 44
    "0",                      // 45
    "chatstate",              // 46
    "business_hours_config",  // 47
    "unavailable",            // 48
    "download_buckets",       // 49
    "skmsg",                  // 50
    "verified_level",         // 51
    "composing",              // 52
    "handshake",              // 53
    "device-list",            // 54
    "media",                  // 55
    "text",                   // 56
    "fallback_ip4",           // 57
    "media_conn",             // 58
    "device",                 // 59
    "creation",               // 60
    "location",               // 61
    "config",                 // 62
    "item",                   // 63
    "fallback_ip6",           // 64
    "count",                  // 65
    "w:profile:picture",      // 66
    "image",                  // 67
    "business",               // 68
    "2",                      // 69
    "hostname",               // 70
    "call-creator",           // 71
    "display_name",           // 72
    "relaylatency",           // 73
    "platform",               // 74
    "abprops",                // 75
    "success",                // 76
    "msg",                    // 77
    "offline_preview",        // 78
    "prop",                   // 79
    "key-index",              // 80
    "v",                      // 81
    "day_of_week",            // 82
    "pkmsg",                  // 83
    "version",                // 84
    "1",                      // 85
    "ping",                   // 86
    "w:p",                    // 87
    "download",               // 88
    "video",                  // 89
    "set",                    // 90
    "specific_hours",         // 91
    "props",                  // 92
    "primary",                // 93
    "unknown",                // 94
    "hash",                   // 95
    "commerce_experience",    // 96
    "last",                   // 97
    "subscribe",              // 98
    "max_buckets",            // 99
    "call",                   // 100
    "profile",                // 101
    "member_since_text",      // 102
    "close_time",             // 103
    "call-id",                // 104
    "sticker",                // 105
    "mode",                   // 106
    "participants",           // 107
    "value",                  // 108
    "query",                  // 109
    "profile_options",        // 110
    "open_time",              // 111
    "code",                   // 112
    "list",                   // 113
    "host",                   // 114
    "ts",                     // 115
    "contacts",               // 116
    "upload",                 // 117
    "lid",                    // 118
    "preview",                // 119
    "update",                 // 120
    "usync",                  // 121
    "w:stats",                // 122
    "delivery",               // 123
    "auth_ttl",               // 124
    "context",                // 125
    "fail",                   // 126
    "cart_enabled",           // 127
    "appdata",                // 128
    "category",               // 129
    "atn",                    // 130
    "direct_connection",      // 131
    "decrypt-fail",           // 132
    "relay_id",               // 133
    "mmg-fallback.whatsapp.net", // 134
    "target",                 // 135
    "available",              // 136
    "name",                   // 137
    "last_id",                // 138
    "mmg.whatsapp.net",       // 139
    "categories",             // 140
    "401",                    // 141
    "is_new",                 // 142
    "index",                  // 143
    "tctoken",                // 144
    "ip4",                    // 145
    "token_id",               // 146
    "latency",                // 147
    "recipient",              // 148
    "edit",                   // 149
    "ip6",                    // 150
    "add",                    // 151
    "thumbnail-document",     // 152
    "26",                     // 153
    "paused",                 // 154
    "true",                   // 155
    "identity",               // 156
    "stream:error",           // 157
    "key",                    // 158
    "sidelist",               // 159
    "background",             // 160
    "audio",                  // 161
    "3",                      // 162
    "thumbnail-image",        // 163
    "biz-cover-photo",        // 164
    "cat",                    // 165
    "gcm",                    // 166
    "thumbnail-video",        // 167
    "error",                  // 168
    "auth",                   // 169
    "deny",                   // 170
    "serial",                 // 171
    "in",                     // 172
    "registration",           // 173
    "thumbnail-link",         // 174
    "remove",                 // 175
    "00",                     // 176
    "gif",                    // 177
    "thumbnail-gif",          // 178
    "tag",                    // 179
    "capability",             // 180
    "multicast",              // 181
    "item-not-found",         // 182
    "description",            // 183
    "business_hours",         // 184
    "config_expo_key",        // 185
    "md-app-state",           // 186
    "expiration",             // 187
    "fallback",               // 188
    "ttl",                    // 189
    "300",                    // 190
    "md-msg-hist",            // 191
    "device_orientation",     // 192
    "out",                    // 193
    "w:m",                    // 194
    "open_24h",               // 195
    "side_list",              // 196
    "token",                  // 197
    "inactive",               // 198
    "01",                     // 199
    "document",               // 200
    "te2",                    // 201
    "played",                 // 202
    "encrypt",                // 203
    "msgr",                   // 204
    "hide",                   // 205
    "direct_path",            // 206
    "12",                     // 207
    "state",                  // 208
    "not-authorized",         // 209
    "url",                    // 210
    "terminate",              // 211
    "signature",              // 212
    "status-revoke-delay",    // 213
    "02",                     // 214
    "te",                     // 215
    "linked_accounts",        // 216
    "trusted_contact",        // 217
    "timezone",               // 218
    "ptt",                    // 219
    "kyc-id",                 // 220
    "privacy_token",          // 221
    "readreceipts",           // 222
    "appointment_only",       // 223
    "address",                // 224
    "expected_ts",            // 225
    "privacy",                // 226
    "7",                      // 227
    "android",                // 228
    "interactive",            // 229
    "device-identity",        // 230
    "enabled",                // 231
    "attribute_padding",      // 232
    "1080",                   // 233
    "03",                     // 234
    "screen_height",          // 235
];

use std::collections::HashMap;
use std::sync::OnceLock;

/// Get the token index for a string (reverse lookup)
pub fn get_token_index(s: &str) -> Option<u8> {
    static TOKEN_MAP: OnceLock<HashMap<&'static str, u8>> = OnceLock::new();
    
    let map = TOKEN_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for (i, token) in SINGLE_BYTE_TOKENS.iter().enumerate() {
            if !token.is_empty() {
                m.insert(*token, i as u8);
            }
        }
        m
    });
    
    map.get(s).copied()
}

/// Get the string for a token index
pub fn get_token(index: u8) -> Option<&'static str> {
    SINGLE_BYTE_TOKENS.get(index as usize).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_lookup() {
        assert_eq!(get_token(3), Some("s.whatsapp.net"));
        assert_eq!(get_token(28), Some("g.us"));
        assert_eq!(get_token(19), Some("message"));
    }

    #[test]
    fn test_reverse_lookup() {
        assert_eq!(get_token_index("s.whatsapp.net"), Some(3));
        assert_eq!(get_token_index("g.us"), Some(28));
        assert_eq!(get_token_index("message"), Some(19));
    }

    #[test]
    fn test_unknown_token() {
        assert_eq!(get_token_index("unknown_string_xyz"), None);
    }
}
