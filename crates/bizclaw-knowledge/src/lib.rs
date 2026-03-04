//! # BizClaw Knowledge Base
//!
//! Personal RAG (Retrieval-Augmented Generation) with hybrid search.
//! Works on 512MB RAM devices — no external vector DB needed.
//!
//! ## Design
//! - **SQLite FTS5** for keyword search (BM25 scoring)
//! - **Vector embeddings** via Ollama or OpenAI (stored as BLOB in SQLite)
//! - **Hybrid search** — keyword (0.3) + vector similarity (0.7)
//! - **Chunking** — split documents into ~500 char chunks
//! - **File-based** — documents stored as-is, index in SQLite
//! - RAM: ~2MB for 1000 document chunks
//!
//! ## How it works
//! ```text
//! User: "Chính sách làm việc từ xa ra sao?"
//!   ↓
//! Knowledge.search("chính sách làm việc từ xa")
//!   ↓ FTS5 + BM25 + Vector Cosine Similarity
//! Top 3 chunks from uploaded documents
//!   ↓
//! Injected into Agent system prompt as context
//!   ↓
//! Agent responds with grounded answer
//! ```

pub mod chunker;
pub mod embeddings;
pub mod search;
pub mod store;
pub mod vector_store;

#[cfg(feature = "pdf")]
pub mod pdf;

pub use search::SearchResult;
pub use store::KnowledgeStore;
