DELETE FROM permissions WHERE resource = 'platform' AND action IN ('read', 'update');
