-- 移除記帳本 / 發票對獎 / 樂透對獎三個功能：資料表、會員通知欄位、權限與設定值
DROP TABLE IF EXISTS public.invoices;
DROP TABLE IF EXISTS public.invoice_lottery_numbers;
DROP TABLE IF EXISTS public.lotto_tickets;
DROP TABLE IF EXISTS public.lotto_draws;
DROP TABLE IF EXISTS public.ledger_entries;

ALTER TABLE public.members DROP COLUMN IF EXISTS lottery_notify_enabled;
ALTER TABLE public.members DROP COLUMN IF EXISTS lotto_notify_enabled;

-- role_permissions 有 ON DELETE CASCADE，掛在此權限上的授權一併消失
DELETE FROM public.permissions WHERE resource = 'invoice_lottery' AND action = 'write';

-- 從 instance 功能開關與首頁卡片清單移除三個 key（值為 'all' 時不是陣列，跳過）
UPDATE public.app_settings
SET value = (
    SELECT COALESCE(jsonb_agg(k)::text, '[]')
    FROM jsonb_array_elements_text(value::jsonb) AS k
    WHERE k NOT IN ('ledger', 'invoices', 'lotto')
)
WHERE key IN ('enabled_features', 'home_features')
  AND value ~ '^\s*\[';
