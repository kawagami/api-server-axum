CREATE TABLE portfolio (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id      BIGINT NOT NULL,
    stock_code     TEXT NOT NULL,
    buy_date       DATE NOT NULL,
    cost_per_share DOUBLE PRECISION NOT NULL,
    shares         BIGINT NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
