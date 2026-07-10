-- 移除 HackMD 筆記同步功能:資料表與 runtime 設定
DROP TABLE IF EXISTS public.hackmd_posts;
DROP TABLE IF EXISTS public.hackmd_users;

DELETE FROM public.app_settings WHERE key = 'hackmd_token';
