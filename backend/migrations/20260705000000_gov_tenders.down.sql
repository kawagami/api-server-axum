DELETE FROM role_permissions
WHERE permission_id IN (SELECT id FROM permissions WHERE resource = 'gov_tender');
DELETE FROM permissions WHERE resource = 'gov_tender';
DELETE FROM app_settings WHERE key = 'gov_tender_keywords';
DROP TABLE gov_tenders;
