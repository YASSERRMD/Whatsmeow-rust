//! Lightweight, documented scaffolding for a Rust port of the Whatsmeow client.
//!
//! The upstream Whatsmeow project is a full-featured Go library for interacting
//! with WhatsApp. This crate does not attempt to mirror every feature. Instead,
//! it exposes a small set of building blocks—configuration, client state, and a
//! simple client façade—that can be extended into a larger implementation.
//!
//! Networking, QR login, encryption, and persistence are implemented as local
//! simulations to mirror upstream concerns. Replace them with production-grade
//! protocol implementations before building on this scaffold.

pub mod barq_api;
pub mod barq_core;
pub mod client;
pub mod config;
pub mod state;

pub use barq_api::{
    ExplainScore, HybridSearchRequest, KeywordSearchRequest, SearchResponse, SearchWeights,
};
pub use barq_core::{
    Bm25Config, Collection, Document, EnglishAnalyzer, ScoredDoc, TextField, bm25_search,
    cosine_similarity, vector_search,
};
pub use client::{ClientError, WhatsmeowClient};
pub use config::WhatsmeowConfig;
pub use state::{MediaItem, MessageStatus, NetworkState, QrLogin, SessionState};
