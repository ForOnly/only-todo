-- 桌面伴侣：双窗 + 默认形态当家。从 floating_window.* 一次性迁移。
INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.home_shape', CASE value WHEN 'panel' THEN 'panel' ELSE 'ball' END
FROM settings WHERE key = 'floating_window.default_mode';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.home_shape', 'ball');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.placement', CASE value WHEN 'docked' THEN 'docked' ELSE 'free' END
FROM settings WHERE key = 'floating_window.display_mode';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.placement', 'free');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.panel_bounds', value FROM settings WHERE key = 'floating_window.panel_bounds';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.panel_bounds', '{"x":100,"y":100,"width":320,"height":480}');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.ball_pos', json_object('x', json_extract(value, '$.x'), 'y', json_extract(value, '$.y'), 'width', 64, 'height', 64)
FROM settings WHERE key = 'floating_window.ball_bounds';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.ball_pos', '{"x":100,"y":100,"width":64,"height":64}');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.always_on_top', value FROM settings WHERE key = 'floating_window.always_on_top';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.always_on_top', 'true');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.visible_count', value FROM settings WHERE key = 'floating_window.visible_count';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.visible_count', '5');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.auto_show', value FROM settings WHERE key = 'floating_window.auto_show';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.auto_show', 'true');

INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.hover_preview', 'false');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.dock_edge', CASE value WHEN 'left' THEN 'left' ELSE 'right' END
FROM settings WHERE key = 'floating_window.dock_edge';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.dock_edge', 'right');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.dock_y', CAST(json_extract(value, '$.y') AS TEXT)
FROM settings WHERE key = 'floating_window.ball_bounds';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.dock_y', '100');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.has_docked', CASE value WHEN 'docked' THEN 'true' ELSE 'false' END
FROM settings WHERE key = 'floating_window.display_mode';
INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.has_docked', CASE value WHEN 'docked' THEN 'true' ELSE 'false' END
FROM settings WHERE key = 'companion.placement';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.has_docked', 'false');
