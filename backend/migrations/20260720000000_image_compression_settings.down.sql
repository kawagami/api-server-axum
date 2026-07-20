DELETE FROM public.app_settings
WHERE key IN ('image_webp_quality', 'image_client_compress', 'image_client_quality', 'image_client_max_edge');
