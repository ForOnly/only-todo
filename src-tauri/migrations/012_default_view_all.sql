-- 剥离焦点台：旧默认落点 today/doing/overdue/tag → all
UPDATE settings
SET value = 'all'
WHERE key = 'list.default_view'
  AND value IN ('today', 'doing', 'overdue', 'tag');
