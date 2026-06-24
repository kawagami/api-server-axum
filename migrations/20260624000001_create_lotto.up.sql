-- 大樂透 / 威力彩 開獎結果（全站共用、無 member）
CREATE TABLE lotto_draws (
    id          SERIAL PRIMARY KEY,
    game        TEXT NOT NULL,            -- 'lotto649' | 'super_lotto638'
    period      TEXT NOT NULL,            -- 台彩期別字串（如 '115000057'），資訊用
    draw_date   DATE NOT NULL,            -- 開獎日，對獎的對應鍵
    main_nums   SMALLINT[] NOT NULL,      -- 一般號 / 第一區，6 個（已排序）
    special     SMALLINT NOT NULL,        -- 大樂透=特別號；威力彩=第二區號
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (game, draw_date)              -- 抓取冪等：同遊戲同開獎日只一筆
);

-- member 登錄的注（一注一列）
CREATE TABLE lotto_tickets (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id   BIGINT NOT NULL,
    game        TEXT NOT NULL,            -- 'lotto649' | 'super_lotto638'
    draw_date   DATE NOT NULL,            -- 這注要對的開獎日（對應 lotto_draws.draw_date）
    picks       SMALLINT[] NOT NULL,      -- 一般號 / 第一區，6 個
    second      SMALLINT,                 -- 威力彩第二區號；大樂透為 NULL
    source      TEXT NOT NULL DEFAULT 'manual',  -- 'manual' | 'qr'
    checked     BOOLEAN NOT NULL DEFAULT false,   -- 該期是否已開獎並比對過
    prize_tier  TEXT,                     -- 命中獎別 key；checked=true 且 null = 確定未中
    notified_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lotto_tickets_member ON lotto_tickets (member_id, created_at DESC);
CREATE INDEX idx_lotto_tickets_pending ON lotto_tickets (game, draw_date) WHERE checked = false;

-- member 樂透中獎 email 通知偏好（預設關閉，須主動開啟且有 email；與發票通知獨立）
ALTER TABLE members ADD COLUMN lotto_notify_enabled BOOLEAN NOT NULL DEFAULT false;
