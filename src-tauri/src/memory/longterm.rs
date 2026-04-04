/// Long-term memory — persistent patterns stored in ~/.grove/memory/patterns/.
/// Future: migrate to vector DB (Qdrant) for semantic search.

// TODO (Session 2): Implement pattern storage and retrieval.
// - Store as individual markdown files in patterns/ directory.
// - Each pattern has: description, confidence, occurrences, first/last seen.
// - Promote from working memory when pattern detected 3+ times.
