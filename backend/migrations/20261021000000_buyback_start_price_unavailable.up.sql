-- TWSE STOCK_DAY_AVG 明確回「查無資料」的庫藏股期間（上櫃股、轉上市前的月份）。
-- FetchHistoricalClosingPrices 每分鐘只取最舊一筆缺起始日價格的期間，沒有這個標記的話
-- 查無資料的那筆會每分鐘被重撿，排在後面的期間全部補不到，直到它的 end_date 過期。
-- 只有「查無資料」才寫；狀態碼錯誤 / decode 失敗等可能是暫時的，不標記、下一分鐘重試。
-- bulk_insert_stock_buyback_periods 的 ON CONFLICT 只更新 end_date，每日同步不會清掉這欄。
ALTER TABLE stock_buyback_periods
    ADD COLUMN start_price_unavailable_at TIMESTAMPTZ;
