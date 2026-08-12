-- 若本机已执行过实验性 007，用 INSERT OR IGNORE 补齐缺键（含 home_shape / ball_pos）
INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.home_shape', CASE value WHEN 'panel' THEN 'panel' ELSE 'ball' END
FROM settings WHERE key IN ('floating_window.default_mode', 'companion.home_shape');
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.home_shape', 'ball');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.placement', CASE value WHEN 'docked' THEN 'docked' ELSE 'free' END
FROM settings WHERE key IN ('floating_window.display_mode', 'companion.placement');
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.placement', 'free');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.panel_bounds', value FROM settings WHERE key IN ('floating_window.panel_bounds', 'companion.panel_bounds');
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.panel_bounds', '{"x":100,"y":100,"width":320,"height":480}');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.ball_pos', json_object('x', json_extract(value, '$.x'), 'y', json_extract(value, '$.y'), 'width', 64, 'height', 64)
FROM settings WHERE key IN ('floating_window.ball_bounds', 'companion.ball_pos');
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.ball_pos', '{"x":100,"y":100,"width":64,"height":64}');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.always_on_top', value FROM settings WHERE key IN ('floating_window.always_on_top', 'companion.always_on_top');
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.always_on_top', 'true');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.visible_count', value FROM settings WHERE key IN ('floating_window.visible_count', 'companion.visible_count');
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.visible_count', '5');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.auto_show', value FROM settings WHERE key IN ('floating_window.auto_show', 'companion.auto_show');
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.auto_show', 'true');

INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.hover_preview', 'false');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.dock_edge', CASE value WHEN 'left' THEN 'left' ELSE 'right' END
FROM settings WHERE key IN ('floating_window.dock_edge', 'companion.dock_edge');
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.dock_edge', 'right');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.dock_y', value FROM settings WHERE key = 'companion.dock_y';
INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.dock_y', CAST(json_extract(value, '$.y') AS TEXT)
FROM settings WHERE key = 'floating_window.ball_bounds';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.dock_y', '100');

INSERT OR IGNORE INTO settings (key, value)
SELECT 'companion.has_docked', CASE value WHEN 'docked' THEN 'true' ELSE 'false' END
FROM settings WHERE key = 'companion.placement';
INSERT OR IGNORE INTO settings (key, value) VALUES ('companion.has_docked', 'false');
