CREATE TABLE IF NOT EXISTS avoid_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    reason TEXT NOT NULL,
    alternative TEXT,
    severity TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_avoid_items_created_at ON avoid_items(created_at DESC);
