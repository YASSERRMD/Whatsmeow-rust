use std::collections::HashMap;
use std::thread;

use crate::barq_core::{
    Bm25Config, Collection, EnglishAnalyzer, ScoredDoc, TextField, bm25_search, vector_search,
};

/// Incoming keyword-only search request over BM25.
#[derive(Debug, Clone, PartialEq)]
pub struct KeywordSearchRequest {
    pub query: String,
    pub top_k: usize,
    pub bm25: Bm25Config,
}

impl KeywordSearchRequest {
    pub fn new(query: impl Into<String>, top_k: usize) -> Self {
        Self {
            query: query.into(),
            top_k,
            bm25: Bm25Config::default(),
        }
    }
}

/// Hybrid search request combining BM25 with vector similarity.
#[derive(Debug, Clone, PartialEq)]
pub struct HybridSearchRequest {
    pub query: String,
    pub query_embedding: Vec<f32>,
    pub top_k: usize,
    pub bm25: Bm25Config,
    pub weights: SearchWeights,
}

impl HybridSearchRequest {
    pub fn new(query: impl Into<String>, query_embedding: Vec<f32>, top_k: usize) -> Self {
        Self {
            query: query.into(),
            query_embedding,
            top_k,
            bm25: Bm25Config::default(),
            weights: SearchWeights::default(),
        }
    }
}

/// Relative weights applied when combining normalized scores.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchWeights {
    pub text_weight: f32,
    pub vector_weight: f32,
}

impl Default for SearchWeights {
    fn default() -> Self {
        Self {
            text_weight: 0.5,
            vector_weight: 0.5,
        }
    }
}

/// Aggregated explainability for a hybrid result.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainScore {
    pub doc_id: String,
    pub bm25_score: f32,
    pub vector_score: f32,
    pub combined_score: f32,
}

/// Response returned by keyword-only or hybrid search endpoints.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResponse {
    pub results: Vec<ExplainScore>,
}

/// Mark a field as indexed and rebuild the text analyzer tokens.
pub fn index_text_field(value: impl Into<String>, indexed: bool) -> TextField {
    TextField::new(value, indexed)
}

/// Keyword-only BM25 search endpoint.
pub fn keyword_search(collection: &Collection, request: KeywordSearchRequest) -> SearchResponse {
    let bm25_results = bm25_search(collection, &request.query, request.bm25, request.top_k);
    SearchResponse {
        results: bm25_results
            .into_iter()
            .map(|doc| ExplainScore {
                doc_id: doc.doc_id,
                bm25_score: doc.score,
                vector_score: 0.0,
                combined_score: doc.score,
            })
            .collect(),
    }
}

/// Hybrid search endpoint that runs BM25 and vector search in parallel and combines scores.
pub fn hybrid_search(collection: &Collection, request: HybridSearchRequest) -> SearchResponse {
    let (bm25_results, vector_results) = thread::scope(|s| {
        let collection_for_text = collection.clone();
        let collection_for_vector = collection.clone();
        let query = request.query.clone();
        let query_embedding = request.query_embedding.clone();

        let text_handle =
            s.spawn(move || bm25_search(&collection_for_text, &query, request.bm25, request.top_k));
        let vector_handle =
            s.spawn(move || vector_search(&collection_for_vector, &query_embedding, request.top_k));

        (
            text_handle
                .join()
                .unwrap_or_else(|_| Vec::<ScoredDoc>::new()),
            vector_handle
                .join()
                .unwrap_or_else(|_| Vec::<ScoredDoc>::new()),
        )
    });

    let normalized_bm25 = normalize_scores(&bm25_results);
    let normalized_vector = normalize_scores(&vector_results);

    let mut combined: HashMap<String, ExplainScore> = HashMap::new();

    for doc in bm25_results {
        let norm = *normalized_bm25.get(&doc.doc_id).unwrap_or(&0.0);
        let vector_norm = *normalized_vector.get(&doc.doc_id).unwrap_or(&0.0);
        let combined_score =
            norm * request.weights.text_weight + vector_norm * request.weights.vector_weight;
        let doc_id = doc.doc_id.clone();
        combined.insert(
            doc_id.clone(),
            ExplainScore {
                doc_id,
                bm25_score: doc.score,
                vector_score: *normalized_vector.get(&doc.doc_id).unwrap_or(&0.0),
                combined_score,
            },
        );
    }

    for doc in vector_results {
        let norm = *normalized_bm25.get(&doc.doc_id).unwrap_or(&0.0);
        let vector_norm = *normalized_vector.get(&doc.doc_id).unwrap_or(&0.0);
        let combined_score =
            norm * request.weights.text_weight + vector_norm * request.weights.vector_weight;
        combined
            .entry(doc.doc_id.clone())
            .and_modify(|existing| {
                existing.vector_score = doc.score;
                existing.combined_score = combined_score;
            })
            .or_insert_with(|| ExplainScore {
                doc_id: doc.doc_id.clone(),
                bm25_score: *normalized_bm25.get(&doc.doc_id).unwrap_or(&0.0),
                vector_score: doc.score,
                combined_score,
            });
    }

    let mut results: Vec<ExplainScore> = combined.into_values().collect();
    results.sort_by(|a, b| {
        b.combined_score
            .partial_cmp(&a.combined_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(request.top_k);

    SearchResponse { results }
}

/// Explain a specific document from an existing response.
pub fn explain(response: &SearchResponse, doc_id: &str) -> Option<ExplainScore> {
    response
        .results
        .iter()
        .find(|r| r.doc_id == doc_id)
        .cloned()
}

fn normalize_scores(results: &[ScoredDoc]) -> HashMap<String, f32> {
    if results.is_empty() {
        return HashMap::new();
    }
    let min = results
        .iter()
        .map(|r| r.score)
        .fold(f32::INFINITY, f32::min);
    let max = results
        .iter()
        .map(|r| r.score)
        .fold(f32::NEG_INFINITY, f32::max);
    let range = (max - min).max(std::f32::EPSILON);
    results
        .iter()
        .map(|r| (r.doc_id.clone(), (r.score - min) / range))
        .collect()
}

/// Convenience helper to quickly assemble a collection with indexed text fields.
pub fn build_indexed_collection(
    name: &str,
    documents: Vec<(String, Vec<(String, String)>, Option<Vec<f32>>)>,
) -> Collection {
    let analyzer = EnglishAnalyzer::new();
    let mut collection = Collection::new(name);

    for (doc_id, fields, embedding) in documents {
        let mut doc = super::barq_core::Document::new(doc_id);
        for (field_name, value) in fields {
            let tokens = analyzer.analyze(&value);
            let is_indexed = !tokens.is_empty();
            doc = doc.with_text_field(field_name, TextField::new(value, is_indexed));
        }
        if let Some(embedding) = embedding {
            doc = doc.with_embedding(embedding);
        }
        collection.add_document(doc);
    }

    collection
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::barq_core::Document;

    #[test]
    fn keyword_search_ranks_text_matches() {
        let mut collection = Collection::new("text-only");
        collection.add_document(
            Document::new("doc-1")
                .with_text_field("title", TextField::new("Rust search guide", true)),
        );
        collection.add_document(
            Document::new("doc-2").with_text_field("title", TextField::new("Cooking pasta", true)),
        );

        let response = keyword_search(
            &collection,
            KeywordSearchRequest {
                query: "rust search".into(),
                top_k: 2,
                bm25: Bm25Config::default(),
            },
        );

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].doc_id, "doc-1");
        assert!(response.results[0].bm25_score > 0.0);
    }

    #[test]
    fn hybrid_search_combines_vector_and_text_scores() {
        let collection = build_indexed_collection(
            "hybrid",
            vec![
                (
                    "doc-1".into(),
                    vec![("title".into(), "Rust search tutorial".into())],
                    Some(vec![0.9, 0.1]),
                ),
                (
                    "doc-2".into(),
                    vec![("title".into(), "Vector database article".into())],
                    Some(vec![0.1, 0.9]),
                ),
            ],
        );

        let response = hybrid_search(
            &collection,
            HybridSearchRequest {
                query: "rust search".into(),
                query_embedding: vec![0.8, 0.2],
                top_k: 2,
                bm25: Bm25Config::default(),
                weights: SearchWeights {
                    text_weight: 0.6,
                    vector_weight: 0.4,
                },
            },
        );

        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].doc_id, "doc-1");
        assert!(
            response
                .results
                .iter()
                .any(|r| r.vector_score > 0.0 && r.bm25_score > 0.0)
        );

        let explain = explain(&response, "doc-1").expect("doc present");
        assert!(explain.combined_score > 0.0);
    }
}
