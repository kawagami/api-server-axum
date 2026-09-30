-- 除權息從來沒有成功寫進來過：TWT49U 現行欄位是「資料日期、股票代號、股票名稱、除權息前收盤價…」、
-- 日期是「115年09月29日」，舊解析的欄位索引與日期格式兩者都對不上，每一列都被濾掉。
-- 於是 stock_ex_rights 一直是空的，stock_ex_rights_checked 的每一筆都是「沒解析到任何東西
-- 卻標記為已確認」的假紀錄 —— 兩張整批清掉，讓 portfolio 重新向 TWSE 抓。
DELETE FROM stock_ex_rights_checked;
DELETE FROM stock_ex_rights;

-- 還原因子改用 TWSE 算好的「減除股利參考價」/「除權息前收盤價」：現金股利與無償配股都涵蓋，
-- TWT49U 本來就沒有獨立的配股率欄位，自己拆 cash_div / stock_rate 拆不出來。
ALTER TABLE stock_ex_rights
    DROP COLUMN cash_div,
    DROP COLUMN stock_rate,
    ADD COLUMN ref_price DOUBLE PRECISION NOT NULL;

-- 已向 TWSE 確認到哪一天（含）。舊設計只有 checked_at + 「30 天內可信」，DB 只要有任何一筆
-- 除權息就不再往後查，之後的配息永遠補不進來；改成記涵蓋範圍、每次從這裡接著往後查。
ALTER TABLE stock_ex_rights_checked
    ADD COLUMN covered_until DATE NOT NULL;
