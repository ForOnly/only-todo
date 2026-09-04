-- 出厂助理列表条数 5 → 10；仅升级仍为旧默认 5 的配置
UPDATE settings SET value = '10'
WHERE key = 'companion.visible_count' AND value = '5';
