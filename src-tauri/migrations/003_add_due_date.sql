ALTER TABLE todos ADD COLUMN due_date TEXT;
CREATE INDEX IF NOT EXISTS idx_todos_due_date ON todos(due_date) WHERE deleted_at IS NULL;

INSERT OR IGNORE INTO settings (key, value) VALUES ('floating_window.bounds', '{"x":100,"y":100,"width":320,"height":480}');
INSERT OR IGNORE INTO settings (key, value) VALUES ('floating_window.always_on_top', 'true');
INSERT OR IGNORE INTO settings (key, value) VALUES ('floating_window.visible_count', '5');
INSERT OR IGNORE INTO settings (key, value) VALUES ('floating_window.auto_show', 'true');
