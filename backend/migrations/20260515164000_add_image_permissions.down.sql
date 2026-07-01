DELETE FROM permissions WHERE resource = 'image' AND action IN ('read', 'write', 'delete');
