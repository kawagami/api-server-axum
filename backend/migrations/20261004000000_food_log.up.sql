-- 會員飲食日記：以「吃了什麼、花多少」為主，取代手寫的每日紀錄。
-- 刻意沒有食物目錄表：「常吃 / 最近」選項直接從歷史紀錄聚合（只有會員自己的資料量），
-- 打錯的品項改那一筆紀錄就從選項消失，不必另外維護一份目錄。
CREATE TABLE food_log_entries (
    id          BIGSERIAL PRIMARY KEY,
    member_id   BIGINT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    eaten_on    DATE NOT NULL,                  -- 台北日
    meal        TEXT NOT NULL
        CHECK (meal IN ('breakfast', 'lunch', 'dinner', 'snack', 'late_night', 'other')),
    meal_label  TEXT,                           -- 餐別之外的情境（「去教會找阿精」）
    store       TEXT,
    item        TEXT NOT NULL,
    quantity    SMALLINT NOT NULL DEFAULT 1 CHECK (quantity >= 1),
    amount      INTEGER CHECK (amount >= 0),    -- 該行總價（台幣整數）；NULL = 點數兌換 / 請客
    rating      SMALLINT CHECK (rating BETWEEN 1 AND 5),
    note        TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_food_log_entries_member_date ON food_log_entries (member_id, eaten_on DESC);

-- 當天的非飲食備註（跑步、換牙刷刷頭）。沒寫就沒有這一列。
CREATE TABLE food_log_days (
    member_id   BIGINT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    day         DATE NOT NULL,
    note        TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (member_id, day)
);
