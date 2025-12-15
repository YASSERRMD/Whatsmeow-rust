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

/// Double-byte token dictionaries (indices 236-239 use these)
pub static DOUBLE_BYTE_TOKENS: &[&[&str]] = &[
    // Dictionary 0 (tag 236)
    &["read-self", "active", "fbns", "protocol", "reaction", "screen_width", "heartbeat", "deviceid", 
      "2:47DEQpj8", "uploadfieldstat", "voip_settings", "retry", "priority", "longitude", "conflict", 
      "false", "ig_professional", "replaced", "preaccept", "cover_photo", "uncompressed", "encopt", 
      "ppic", "04", "passive", "status-revoke-drop", "keygen", "540", "offer", "rate", "opus", 
      "latitude", "w:gp2", "ver", "4", "business_profile", "medium", "sender", "prev_v_id", "email",
      "website", "invited", "sign_credential", "05", "transport", "skey", "reason", 
      "peer_abtest_bucket", "America/Sao_Paulo", "appid", "refresh", "100", "06", "404", "101", 
      "104", "107", "102", "109", "103", "member_add_mode", "105", "transaction-id", "110", "106",
      "outgoing", "108", "111", "tokens", "followers", "ig_handle", "self_pid", "tue", "dec", "thu",
      "joinable", "peer_pid", "mon", "features", "wed", "peer_device_presence", "pn", "delete", "07",
      "fri", "audio_duration", "admin", "connected", "delta", "rcat", "disable", "collection", "08",
      "480", "sat", "phash", "all", "invite", "accept", "critical_unblock_low", "group_update",
      "signed_credential", "blinded_credential", "eph_setting", "net", "09", "background_location",
      "refresh_id", "Asia/Kolkata", "privacy_mode_ts", "account_sync", "voip_payload_type",
      "service_areas", "acs_public_key", "v_id", "0a", "fallback_class", "relay", "actual_actors",
      "metadata", "w:biz", "5", "connected-limit", "notice", "0b", "host_storage", "fb_page",
      "subject", "privatestats", "invis", "groupadd", "010", "note.m4r", "uuid", "0c", "8000", "sun",
      "372", "1020", "stage", "1200", "720", "canonical", "fb", "011", "video_duration", "0d", "1140",
      "superadmin", "012", "Opening.m4r", "keystore_attestation", "dleq_proof", "013", "timestamp",
      "ab_key", "w:sync:app:state", "0e", "vertical", "600", "p_v_id", "6", "likes", "014", "500",
      "1260", "creator", "0f", "rte", "destination", "group", "group_info",
      "syncd_anti_tampering_fatal_exception_enabled", "015", "dl_bw", "Asia/Jakarta", "vp8/h.264",
      "online", "1320", "fb:multiway", "10", "timeout", "016", "nse_retry",
      "urn:xmpp:whatsapp:dirty", "017", "a_v_id", "web_shops_chat_header_button_enabled", "nse_call",
      "inactive-upgrade", "none", "web", "groups", "2250",
      "mms_hot_content_timespan_in_seconds", "contact_blacklist", "nse_read",
      "suspended_group_deletion_notification", "binary_version", "018",
      "https://www.whatsapp.com/otp/copy/", "reg_push",
      "shops_hide_catalog_attachment_entrypoint", "server_sync", ".",
      "ephemeral_messages_allowed_values", "019", "mms_vcache_aggregation_enabled", "iphone",
      "America/Argentina/Buenos_Aires", "01a", "mms_vcard_autodownload_size_kb", "nse_ver",
      "shops_header_dropdown_menu_item", "dhash", "catalog_status",
      "communities_mvp_new_iqs_serverprop", "blocklist", "default", "11",
      "ephemeral_messages_enabled", "01b", "original_dimensions", "8",
      "mms4_media_retry_notification_encryption_enabled",
      "mms4_server_error_receipt_encryption_enabled", "original_image_url", "sync", "multiway",
      "420", "companion_enc_static", "shops_profile_drawer_entrypoint", "01c",
      "vcard_as_document_size_kb", "status_video_max_duration", "request_image_url", "01d",
      "regular_high", "s_t", "abt", "share_ext_min_preliminary_image_quality", "01e", "32",
      "syncd_key_rotation_enabled", "data_namespace", "md_downgrade_read_receipts2", "patch",
      "polltype", "ephemeral_messages_setting", "userrate", "15",
      "partial_pjpeg_bw_threshold", "played-self", "catalog_exists", "01f", "mute_v2"],
    // Dictionary 1 (tag 237)
    &["reject", "dirty", "announcement", "020", "13", "9", "status_video_max_bitrate",
      "fb:thrift_iq", "offline_batch", "022", "full", "ctwa_first_business_reply_logging",
      "h.264", "smax_id", "group_description_length", "https://www.whatsapp.com/otp/code",
      "status_image_max_edge", "smb_upsell_business_profile_enabled", "021",
      "web_upgrade_to_md_modal", "14", "023", "s_o", "smaller_video_thumbs_status_enabled",
      "media_max_autodownload", "960", "blocking_status", "peer_msg",
      "joinable_group_call_client_version", "group_call_video_maximization_enabled",
      "return_snapshot", "high", "America/Mexico_City",
      "entry_point_block_logging_enabled", "pop", "024", "1050", "16", "1380",
      "one_tap_calling_in_group_chat_size", "regular_low",
      "inline_joinable_education_enabled", "hq_image_max_edge", "locked", "America/Bogota",
      "smb_biztools_deeplink_enabled", "status_image_quality", "1088", "025",
      "payments_upi_intent_transaction_limit", "voip", "w:g2", "027", "md_pin_chat_enabled",
      "026", "multi_scan_pjpeg_download_enabled", "shops_product_grid", "transaction_id"],
    // Dictionary 2 (tag 238)
    &["ctwa_context_enabled", "20", "fna", "hq_image_quality",
      "alt_jpeg_doc_detection_quality", "group_call_max_participants", "pkey", "America/Belem",
      "image_max_kbytes", "web_cart_v1_1_order_message_changes_enabled",
      "ctwa_context_enterprise_enabled", "urn:xmpp:whatsapp:account", "840",
      "Asia/Kuala_Lumpur", "max_participants", "video_remux_after_repair_enabled",
      "stella_addressbook_restriction_type", "660", "900", "780",
      "context_menu_ios13_enabled", "mute-state", "ref", "payments_request_messages", "029",
      "frskmsg", "vcard_max_size_kb", "sample_buffer_gif_player_enabled", "match_last_seen",
      "510", "4983", "video_max_bitrate", "028", "w:comms:chat", "17",
      "frequently_forwarded_max", "groups_privacy_blacklist", "Asia/Karachi", "02a",
      "web_download_document_thumb_mms_enabled", "02b", "hist_sync",
      "biz_block_reasons_version", "1024", "18", "web_is_direct_connection_for_plm_transparent",
      "view_once_write", "file_max_size", "paid_convo_id", "online_privacy_setting",
      "video_max_edge", "view_once_read", "enhanced_storage_management",
      "multi_scan_pjpeg_encoding_enabled", "ctwa_context_forward_enabled",
      "video_transcode_downgrade_enable", "template_doc_mime_types", "hq_image_bw_threshold",
      "30", "body"],
    // Dictionary 3 (tag 239) - abbreviated
    &["stream:features", "regular", "1724", "profile_picture"],
];

/// Get a double-byte token
pub fn get_double_token(dict: u8, index: u8) -> Option<&'static str> {
    DOUBLE_BYTE_TOKENS.get(dict as usize)
        .and_then(|tokens| tokens.get(index as usize))
        .copied()
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
