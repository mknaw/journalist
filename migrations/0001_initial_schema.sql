-- Initial schema for journalist database
-- Create entries table
CREATE TABLE IF NOT EXISTS entries (
    date DATE PRIMARY KEY,
    content TEXT NOT NULL,
    word_count INTEGER NOT NULL DEFAULT 0,
    task_count INTEGER NOT NULL DEFAULT 0,
    event_count INTEGER NOT NULL DEFAULT 0,
    note_count INTEGER NOT NULL DEFAULT 0,
    priority_count INTEGER NOT NULL DEFAULT 0,
    inspiration_count INTEGER NOT NULL DEFAULT 0,
    insight_count INTEGER NOT NULL DEFAULT 0,
    misstep_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create full-text search index
CREATE TABLE IF NOT EXISTS entry_search (
    date DATE,
    content_fts TEXT,
    FOREIGN KEY (date) REFERENCES entries(date) ON DELETE CASCADE
);

-- Create metadata tables
CREATE TABLE IF NOT EXISTS term_frequency (
    term TEXT PRIMARY KEY,
    frequency BIGINT NOT NULL DEFAULT 0,
    first_seen DATE NOT NULL,
    last_seen DATE NOT NULL
);

CREATE TABLE IF NOT EXISTS cross_references (
    source_date DATE,
    target_date DATE,
    reference_type TEXT,
    PRIMARY KEY (source_date, target_date, reference_type),
    FOREIGN KEY (source_date) REFERENCES entries(date) ON DELETE CASCADE,
    FOREIGN KEY (target_date) REFERENCES entries(date) ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date);
CREATE INDEX IF NOT EXISTS idx_entries_updated ON entries(updated_at);
CREATE INDEX IF NOT EXISTS idx_term_frequency_term ON term_frequency(term);
CREATE INDEX IF NOT EXISTS idx_term_frequency_last_seen ON term_frequency(last_seen);