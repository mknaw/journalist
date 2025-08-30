-- Initial schema for journalist database
-- Create bullets table
CREATE TABLE IF NOT EXISTS bullets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATE NOT NULL,
    content TEXT NOT NULL,
    type TEXT NOT NULL, -- task, event, note, priority, inspiration, insight, misstep
    task_state TEXT, -- pending, completed, migrated, scheduled (only for task types)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_bullets_date ON bullets(date);
CREATE INDEX IF NOT EXISTS idx_bullets_type ON bullets(type);
CREATE INDEX IF NOT EXISTS idx_bullets_date_type ON bullets(date, type);
CREATE INDEX IF NOT EXISTS idx_bullets_updated ON bullets(updated_at);

-- Create full-text search index directly on the content column
CREATE INDEX IF NOT EXISTS idx_bullets_content_fts ON bullets USING FTS(content);

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
    PRIMARY KEY (source_date, target_date, reference_type)
);

-- Create indexes for metadata tables
CREATE INDEX IF NOT EXISTS idx_term_frequency_term ON term_frequency(term);
CREATE INDEX IF NOT EXISTS idx_term_frequency_last_seen ON term_frequency(last_seen);