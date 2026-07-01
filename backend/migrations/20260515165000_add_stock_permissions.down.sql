DELETE FROM permissions WHERE resource = 'stock' AND action IN ('read', 'write');
