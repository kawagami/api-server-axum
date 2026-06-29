DELETE FROM permissions WHERE resource = 'stat' AND action = 'read';
DROP TABLE IF EXISTS daily_visitor_stats;
