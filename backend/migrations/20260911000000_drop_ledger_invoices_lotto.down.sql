-- 還原記帳本 / 發票對獎 / 樂透對獎的資料表結構與權限（資料不可還原）
CREATE TABLE public.ledger_entries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    member_id bigint NOT NULL,
    kind text NOT NULL,
    amount numeric(14,2) NOT NULL,
    category text NOT NULL,
    note text,
    occurred_at date NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    invoice_number text,
    seller_tax_id text,
    source text DEFAULT 'manual'::text NOT NULL
);

ALTER TABLE ONLY public.ledger_entries
    ADD CONSTRAINT ledger_entries_pkey PRIMARY KEY (id);

CREATE INDEX idx_ledger_entries_member_date ON public.ledger_entries USING btree (member_id, occurred_at DESC);

CREATE TABLE public.invoices (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    member_id bigint NOT NULL,
    invoice_number text NOT NULL,
    invoice_date date NOT NULL,
    period text NOT NULL,
    amount numeric(14,2),
    seller_tax_id text,
    source text NOT NULL,
    ledger_entry_id uuid,
    lottery_checked boolean DEFAULT false NOT NULL,
    prize_tier text,
    notified_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.invoices
    ADD CONSTRAINT invoices_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.invoices
    ADD CONSTRAINT invoices_ledger_entry_id_fkey FOREIGN KEY (ledger_entry_id) REFERENCES public.ledger_entries(id) ON DELETE SET NULL;

CREATE UNIQUE INDEX idx_invoices_member_number ON public.invoices USING btree (member_id, invoice_number);
CREATE INDEX idx_invoices_period_unchecked ON public.invoices USING btree (period) WHERE (lottery_checked = false);

CREATE TABLE public.invoice_lottery_numbers (
    id integer NOT NULL,
    period text NOT NULL,
    prize_tier text NOT NULL,
    number text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.invoice_lottery_numbers_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.invoice_lottery_numbers_id_seq OWNED BY public.invoice_lottery_numbers.id;
ALTER TABLE ONLY public.invoice_lottery_numbers ALTER COLUMN id SET DEFAULT nextval('public.invoice_lottery_numbers_id_seq'::regclass);

ALTER TABLE ONLY public.invoice_lottery_numbers
    ADD CONSTRAINT invoice_lottery_numbers_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.invoice_lottery_numbers
    ADD CONSTRAINT invoice_lottery_numbers_period_prize_tier_number_key UNIQUE (period, prize_tier, number);

CREATE TABLE public.lotto_draws (
    id integer NOT NULL,
    game text NOT NULL,
    period text NOT NULL,
    draw_date date NOT NULL,
    main_nums smallint[] NOT NULL,
    special smallint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.lotto_draws_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.lotto_draws_id_seq OWNED BY public.lotto_draws.id;
ALTER TABLE ONLY public.lotto_draws ALTER COLUMN id SET DEFAULT nextval('public.lotto_draws_id_seq'::regclass);

ALTER TABLE ONLY public.lotto_draws
    ADD CONSTRAINT lotto_draws_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.lotto_draws
    ADD CONSTRAINT lotto_draws_game_draw_date_key UNIQUE (game, draw_date);

CREATE TABLE public.lotto_tickets (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    member_id bigint NOT NULL,
    game text NOT NULL,
    draw_date date NOT NULL,
    picks smallint[] NOT NULL,
    second smallint,
    source text DEFAULT 'manual'::text NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    prize_tier text,
    notified_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.lotto_tickets
    ADD CONSTRAINT lotto_tickets_pkey PRIMARY KEY (id);

CREATE INDEX idx_lotto_tickets_member ON public.lotto_tickets USING btree (member_id, created_at DESC);
CREATE INDEX idx_lotto_tickets_pending ON public.lotto_tickets USING btree (game, draw_date) WHERE (checked = false);

ALTER TABLE public.members ADD COLUMN lottery_notify_enabled boolean DEFAULT false NOT NULL;
ALTER TABLE public.members ADD COLUMN lotto_notify_enabled boolean DEFAULT false NOT NULL;

INSERT INTO public.permissions (id, resource, action, description)
VALUES (45, 'invoice_lottery', 'write', '手動輸入統一發票中獎號碼')
ON CONFLICT (id) DO NOTHING;
