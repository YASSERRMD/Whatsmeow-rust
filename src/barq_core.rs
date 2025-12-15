use std::collections::HashMap;

use regex::Regex;

/// Token-level statistics tracked for each term in the inverted index.
#[derive(Debug, Clone, PartialEq)]
pub struct Posting {
    pub doc_id: String,
    pub term_freq: usize,
}

/// Inverted index maintained per collection for BM25 scoring.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InvertedIndex {
    pub postings: HashMap<String, Vec<Posting>>,
    pub doc_lengths: HashMap<String, usize>,
    pub doc_count: usize,
}

impl InvertedIndex {
    pub fn index_document(&mut self, doc_id: &str, tokens: &[String]) {
        if !self.doc_lengths.contains_key(doc_id) {
            self.doc_count += 1;
        }

        let mut term_counts: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *term_counts.entry(token.clone()).or_insert(0) += 1;
        }

        for (term, freq) in term_counts {
            let entry = self.postings.entry(term).or_default();
            entry.push(Posting {
                doc_id: doc_id.to_string(),
                term_freq: freq,
            });
        }

        self.doc_lengths.insert(doc_id.to_string(), tokens.len());
    }

    pub fn term_df(&self, term: &str) -> usize {
        self.postings.get(term).map(|p| p.len()).unwrap_or(0)
    }

    pub fn avg_doc_len(&self) -> f32 {
        if self.doc_lengths.is_empty() {
            return 0.0;
        }
        let total: usize = self.doc_lengths.values().sum();
        total as f32 / self.doc_lengths.len() as f32
    }
}

/// Basic analyzer that tokenizes and normalizes English text.
#[derive(Debug, Clone)]
pub struct EnglishAnalyzer {
    token_re: Regex,
}

impl Default for EnglishAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishAnalyzer {
    pub fn new() -> Self {
        Self {
            token_re: Regex::new(r"[A-Za-z0-9]+").expect("valid token regex"),
        }
    }

    pub fn analyze(&self, text: &str) -> Vec<String> {
        self.token_re
            .find_iter(&text.to_lowercase())
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

/// A single text field on a document, optionally indexed for BM25.
#[derive(Debug, Clone, PartialEq)]
pub struct TextField {
    pub value: String,
    pub indexed: bool,
}

impl TextField {
    pub fn new(value: impl Into<String>, indexed: bool) -> Self {
        Self {
            value: value.into(),
            indexed,
        }
    }
}

/// Document stored within a collection with optional embedding for vector search.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub id: String,
    pub fields: HashMap<String, TextField>,
    pub embedding: Option<Vec<f32>>,
}

impl Document {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            fields: HashMap::new(),
            embedding: None,
        }
    }

    pub fn with_text_field(mut self, name: impl Into<String>, field: TextField) -> Self {
        self.fields.insert(name.into(), field);
        self
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

/// Storage for embeddings keyed by document id.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VectorIndex {
    pub vectors: HashMap<String, Vec<f32>>,
}

impl VectorIndex {
    pub fn insert(&mut self, doc_id: &str, embedding: Vec<f32>) {
        self.vectors.insert(doc_id.to_string(), embedding);
    }

    pub fn get(&self, doc_id: &str) -> Option<&Vec<f32>> {
        self.vectors.get(doc_id)
    }
}

/// In-memory collection keeping text and vector indices in sync.
#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub documents: HashMap<String, Document>,
    pub inverted_index: InvertedIndex,
    pub vector_index: VectorIndex,
    analyzer: EnglishAnalyzer,
}

impl Collection {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            documents: HashMap::new(),
            inverted_index: InvertedIndex::default(),
            vector_index: VectorIndex::default(),
            analyzer: EnglishAnalyzer::new(),
        }
    }

    pub fn add_document(&mut self, document: Document) {
        let tokens = self.collect_tokens(&document);
        self.inverted_index.index_document(&document.id, &tokens);

        if let Some(embedding) = &document.embedding {
            self.vector_index.insert(&document.id, embedding.clone());
        }

        self.documents.insert(document.id.clone(), document);
    }

    fn collect_tokens(&self, document: &Document) -> Vec<String> {
        let mut tokens = Vec::new();
        for field in document.fields.values() {
            if field.indexed {
                tokens.extend(self.analyzer.analyze(&field.value));
            }
        }
        tokens
    }
}

/// BM25 configuration parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bm25Config {
    pub k1: f32,
    pub b: f32,
}

impl Default for Bm25Config {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75 }
    }
}

/// A scored document result used by search endpoints.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredDoc {
    pub doc_id: String,
    pub score: f32,
}

/// Compute BM25 scores for a query string against the collection.
pub fn bm25_search(
    collection: &Collection,
    query: &str,
    config: Bm25Config,
    top_k: usize,
) -> Vec<ScoredDoc> {
    let analyzer = EnglishAnalyzer::new();
    let query_terms = analyzer.analyze(query);
    let avg_dl = collection.inverted_index.avg_doc_len();
    let mut scores: HashMap<String, f32> = HashMap::new();

    for term in query_terms {
        let df = collection.inverted_index.term_df(&term) as f32;
        if df == 0.0 {
            continue;
        }
        let idf = ((collection.inverted_index.doc_count as f32 - df + 0.5) / (df + 0.5) + 1.0).ln();
        if let Some(postings) = collection.inverted_index.postings.get(&term) {
            for posting in postings {
                let dl = *collection
                    .inverted_index
                    .doc_lengths
                    .get(&posting.doc_id)
                    .unwrap_or(&0) as f32;
                let tf = posting.term_freq as f32;
                let denom =
                    tf + config.k1 * (1.0 - config.b + config.b * (dl / (avg_dl.max(1e-6))));
                let score = idf * (tf * (config.k1 + 1.0) / denom);
                *scores.entry(posting.doc_id.clone()).or_insert(0.0) += score;
            }
        }
    }

    let mut scored: Vec<ScoredDoc> = scores
        .into_iter()
        .map(|(doc_id, score)| ScoredDoc { doc_id, score })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(top_k);
    scored
}

/// Calculate cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|v| v * v).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Run a simple vector search over stored embeddings.
pub fn vector_search(collection: &Collection, query: &[f32], top_k: usize) -> Vec<ScoredDoc> {
    let mut scored: Vec<ScoredDoc> = collection
        .vector_index
        .vectors
        .iter()
        .filter_map(|(doc_id, emb)| {
            if emb.len() == query.len() {
                Some(ScoredDoc {
                    doc_id: doc_id.clone(),
                    score: cosine_similarity(emb, query),
                })
            } else {
                None
            }
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(top_k);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_inverted_index_with_term_stats() {
        let mut collection = Collection::new("test");
        let doc = Document::new("doc-1")
            .with_text_field("body", TextField::new("Rust search example", true));
        collection.add_document(doc);

        assert_eq!(collection.inverted_index.doc_count, 1);
        assert_eq!(collection.inverted_index.doc_lengths.get("doc-1"), Some(&3));
        assert_eq!(collection.inverted_index.term_df("rust"), 1);
        assert_eq!(collection.inverted_index.term_df("missing"), 0);
    }

    #[test]
    fn bm25_prioritizes_relevant_documents() {
        let mut collection = Collection::new("search");
        collection.add_document(
            Document::new("doc-relevant")
                .with_text_field("body", TextField::new("Rust language search scoring", true)),
        );
        collection.add_document(
            Document::new("doc-noise")
                .with_text_field("body", TextField::new("Cooking recipe and spices", true)),
        );

        let results = bm25_search(&collection, "rust search", Bm25Config::default(), 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, "doc-relevant");
        if results.len() > 1 {
            assert!(results[0].score >= results.last().unwrap().score);
        }
    }
}
