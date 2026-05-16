DELETE FROM permissions WHERE (resource = 'image' AND action = 'write')
                          OR (resource = 'stock' AND action = 'write');
