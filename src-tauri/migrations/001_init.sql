CREATE TABLE IF NOT EXISTS schema_version (
    version     INTEGER PRIMARY KEY,
    applied_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS todos (
    id           TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'Todo'
                 CHECK (status IN ('Todo','Doing','Done','Archived')),
    priority     TEXT NOT NULL DEFAULT 'Medium'
                 CHECK (priority IN ('Low','Medium','High','Urgent')),
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    completed_at TEXT,
    deleted_at   TEXT
);

CREATE INDEX IF NOT EXISTS idx_todos_status ON todos(status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_todos_priority ON todos(priority) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_todos_created_at ON todos(created_at);

CREATE TABLE IF NOT EXISTS reminders (
    id              TEXT PRIMARY KEY,
    todo_id         TEXT NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
    remind_at       TEXT NOT NULL,
    repeat_type     TEXT NOT NULL DEFAULT 'none'
                    CHECK (repeat_type IN ('none','daily','weekly')),
    repeat_config   TEXT NOT NULL DEFAULT '{}',
    snooze_count    INTEGER NOT NULL DEFAULT 0,
    next_trigger_at TEXT NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reminders_next_trigger ON reminders(next_trigger_at) WHERE enabled = 1;
CREATE INDEX IF NOT EXISTS idx_reminders_todo_id ON reminders(todo_id);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
