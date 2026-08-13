-- 规范化标签：独立 tags 表 + todo_tags 关联（todos.tags JSON 仍作快照）
CREATE TABLE IF NOT EXISTS tags (
    name TEXT PRIMARY KEY COLLATE NOCASE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS todo_tags (
    todo_id TEXT NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
    tag_name TEXT NOT NULL COLLATE NOCASE,
    PRIMARY KEY (todo_id, tag_name),
    FOREIGN KEY (tag_name) REFERENCES tags(name) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_todo_tags_tag ON todo_tags(tag_name);
