DELETE FROM permissions WHERE resource = 'user' AND action IN ('create', 'delete');
