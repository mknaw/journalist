CREATE SEQUENCE IF NOT EXISTS bullet_id_seq;

CREATE TABLE IF NOT EXISTS bullets (
    id INTEGER PRIMARY KEY DEFAULT nextval('bullet_id_seq'),
    date DATE NOT NULL,
    content TEXT NOT NULL,
    type TEXT NOT NULL, -- task, event, note, priority, inspiration, insight, misstep
    task_state TEXT, -- pending, completed, migrated, scheduled (only for task types)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_bullets_date ON bullets(date);

PRAGMA create_fts_index('bullets', 'id', 'content');
