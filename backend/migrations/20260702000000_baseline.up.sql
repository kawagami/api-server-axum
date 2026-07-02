-- Baseline schema：squash 自 20241001000000_create_users_table ～ 20260630000001_visitor_stats
-- 共 60 個歷史 migration（完整歷史見 git）。由 pg_dump (PostgreSQL 18) 產生後整理。

-- Name: update_timestamp(); Type: FUNCTION; Schema: public; Owner: -

CREATE FUNCTION public.update_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

-- Name: admin_audit_logs; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.admin_audit_logs (
    id bigint NOT NULL,
    user_email character varying(255) NOT NULL,
    method character varying(10) NOT NULL,
    path text NOT NULL,
    query text,
    status_code smallint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: admin_audit_logs_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.admin_audit_logs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: admin_audit_logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.admin_audit_logs_id_seq OWNED BY public.admin_audit_logs.id;

-- Name: app_settings; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.app_settings (
    key text NOT NULL,
    value text DEFAULT ''::text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    category text DEFAULT 'general'::text NOT NULL
);

-- Name: blogs; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.blogs (
    id uuid NOT NULL,
    markdown text NOT NULL,
    tocs text[] DEFAULT '{}'::text[],
    tags text[] DEFAULT '{}'::text[],
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

-- Name: chat_messages; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.chat_messages (
    id integer NOT NULL,
    message_type character varying(16) NOT NULL,
    to_type character varying(16) NOT NULL,
    user_name character varying(16) NOT NULL,
    message text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

-- Name: chat_messages_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.chat_messages_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: chat_messages_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.chat_messages_id_seq OWNED BY public.chat_messages.id;

-- Name: daily_visitor_stats; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.daily_visitor_stats (
    date date NOT NULL,
    unique_visitors bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: firebase_images; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.firebase_images (
    id integer NOT NULL,
    image_url text NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

-- Name: firebase_images_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.firebase_images_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: firebase_images_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.firebase_images_id_seq OWNED BY public.firebase_images.id;

-- Name: hackmd_posts; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.hackmd_posts (
    id text NOT NULL,
    content text,
    created_at bigint NOT NULL,
    last_changed_at bigint NOT NULL,
    permalink text,
    publish_link text,
    publish_type text NOT NULL,
    published_at bigint,
    read_permission text NOT NULL,
    short_id text NOT NULL,
    tags text[],
    tags_updated_at bigint,
    team_path text,
    title text NOT NULL,
    title_updated_at bigint NOT NULL,
    user_path text NOT NULL,
    write_permission text NOT NULL
);

-- Name: hackmd_users; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.hackmd_users (
    user_path text NOT NULL,
    biography text,
    name text NOT NULL,
    photo text NOT NULL
);

-- Name: images; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.images (
    id integer NOT NULL,
    storage_key text NOT NULL,
    url text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    status text DEFAULT 'unused'::text NOT NULL
);

-- Name: images_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.images_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: images_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.images_id_seq OWNED BY public.images.id;

-- Name: invoice_lottery_numbers; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.invoice_lottery_numbers (
    id integer NOT NULL,
    period text NOT NULL,
    prize_tier text NOT NULL,
    number text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: invoice_lottery_numbers_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.invoice_lottery_numbers_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: invoice_lottery_numbers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.invoice_lottery_numbers_id_seq OWNED BY public.invoice_lottery_numbers.id;

-- Name: invoices; Type: TABLE; Schema: public; Owner: -

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

-- Name: ledger_entries; Type: TABLE; Schema: public; Owner: -

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

-- Name: logs; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.logs (
    id bigint NOT NULL,
    level character varying(10) NOT NULL,
    message text NOT NULL,
    target character varying(255) NOT NULL,
    file character varying(255),
    line integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: logs_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.logs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.logs_id_seq OWNED BY public.logs.id;

-- Name: lotto_draws; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.lotto_draws (
    id integer NOT NULL,
    game text NOT NULL,
    period text NOT NULL,
    draw_date date NOT NULL,
    main_nums smallint[] NOT NULL,
    special smallint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: lotto_draws_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.lotto_draws_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: lotto_draws_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.lotto_draws_id_seq OWNED BY public.lotto_draws.id;

-- Name: lotto_tickets; Type: TABLE; Schema: public; Owner: -

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

-- Name: member_oauth; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.member_oauth (
    id bigint NOT NULL,
    member_id bigint NOT NULL,
    provider text NOT NULL,
    provider_id text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: member_oauth_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.member_oauth_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: member_oauth_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.member_oauth_id_seq OWNED BY public.member_oauth.id;

-- Name: members; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.members (
    id bigint NOT NULL,
    name text NOT NULL,
    email text,
    avatar_url text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    lottery_notify_enabled boolean DEFAULT false NOT NULL,
    lotto_notify_enabled boolean DEFAULT false NOT NULL
);

-- Name: members_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.members_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: members_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.members_id_seq OWNED BY public.members.id;

-- Name: permissions; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.permissions (
    id integer NOT NULL,
    resource character varying(50) NOT NULL,
    action character varying(50) NOT NULL,
    description text
);

-- Name: permissions_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.permissions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: permissions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.permissions_id_seq OWNED BY public.permissions.id;

-- Name: portfolio; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.portfolio (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    member_id bigint NOT NULL,
    stock_code text NOT NULL,
    buy_date date NOT NULL,
    cost_per_share double precision NOT NULL,
    shares bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: role_permissions; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.role_permissions (
    role_id integer NOT NULL,
    permission_id integer NOT NULL
);

-- Name: roles; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.roles (
    id integer NOT NULL,
    name character varying(50) NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: roles_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.roles_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: roles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.roles_id_seq OWNED BY public.roles.id;

-- Name: stock_buyback_periods; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.stock_buyback_periods (
    id integer NOT NULL,
    stock_no character varying(10) NOT NULL,
    start_date date NOT NULL,
    end_date date NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);

-- Name: TABLE stock_buyback_periods; Type: COMMENT; Schema: public; Owner: -

COMMENT ON TABLE public.stock_buyback_periods IS '執行庫藏股的股票代號與期間資訊';

-- Name: COLUMN stock_buyback_periods.id; Type: COMMENT; Schema: public; Owner: -

COMMENT ON COLUMN public.stock_buyback_periods.id IS '主鍵';

-- Name: COLUMN stock_buyback_periods.stock_no; Type: COMMENT; Schema: public; Owner: -

COMMENT ON COLUMN public.stock_buyback_periods.stock_no IS '股票代號';

-- Name: COLUMN stock_buyback_periods.start_date; Type: COMMENT; Schema: public; Owner: -

COMMENT ON COLUMN public.stock_buyback_periods.start_date IS '庫藏股起始日期（西元）';

-- Name: COLUMN stock_buyback_periods.end_date; Type: COMMENT; Schema: public; Owner: -

COMMENT ON COLUMN public.stock_buyback_periods.end_date IS '庫藏股結束日期（西元）';

-- Name: COLUMN stock_buyback_periods.created_at; Type: COMMENT; Schema: public; Owner: -

COMMENT ON COLUMN public.stock_buyback_periods.created_at IS '建立時間';

-- Name: stock_buyback_periods_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.stock_buyback_periods_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: stock_buyback_periods_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.stock_buyback_periods_id_seq OWNED BY public.stock_buyback_periods.id;

-- Name: stock_changes; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.stock_changes (
    id integer NOT NULL,
    stock_no text NOT NULL,
    start_date date NOT NULL,
    end_date date NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    stock_name text,
    start_price double precision,
    end_price double precision,
    change double precision,
    created_at timestamp with time zone DEFAULT '2026-07-02 12:58:44.098125+00'::timestamp with time zone,
    updated_at timestamp with time zone DEFAULT '2026-07-02 12:58:44.098125+00'::timestamp with time zone
);

-- Name: stock_changes_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.stock_changes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: stock_changes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.stock_changes_id_seq OWNED BY public.stock_changes.id;

-- Name: stock_closing_prices; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.stock_closing_prices (
    id integer NOT NULL,
    stock_no text NOT NULL,
    date date NOT NULL,
    close_price double precision NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);

-- Name: stock_closing_prices_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.stock_closing_prices_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: stock_closing_prices_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.stock_closing_prices_id_seq OWNED BY public.stock_closing_prices.id;

-- Name: stock_day_all; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.stock_day_all (
    id integer NOT NULL,
    trade_date date NOT NULL,
    stock_code text NOT NULL,
    stock_name text NOT NULL,
    trade_volume bigint,
    trade_amount bigint,
    open_price numeric(10,2),
    high_price numeric(10,2),
    low_price numeric(10,2),
    close_price numeric(10,2),
    price_change numeric(10,2),
    transaction_count integer
);

-- Name: stock_day_all_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.stock_day_all_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: stock_day_all_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.stock_day_all_id_seq OWNED BY public.stock_day_all.id;

-- Name: stock_ex_rights; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.stock_ex_rights (
    stock_no text NOT NULL,
    ex_date date NOT NULL,
    close_before double precision DEFAULT 0 NOT NULL,
    cash_div double precision DEFAULT 0 NOT NULL,
    stock_rate double precision DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: stock_ex_rights_checked; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.stock_ex_rights_checked (
    stock_no text NOT NULL,
    from_date date NOT NULL,
    checked_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Name: torrents; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.torrents (
    id integer NOT NULL,
    info_hash text NOT NULL,
    magnet_uri text NOT NULL,
    name text,
    status text DEFAULT 'pending'::text NOT NULL,
    total_size bigint,
    files jsonb,
    error text,
    created_by text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone
);

-- Name: torrents_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.torrents_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: torrents_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.torrents_id_seq OWNED BY public.torrents.id;

-- Name: user_roles; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.user_roles (
    user_id bigint NOT NULL,
    role_id integer NOT NULL
);

-- Name: users; Type: TABLE; Schema: public; Owner: -

CREATE TABLE public.users (
    id bigint NOT NULL,
    name character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    email_verified_at timestamp(0) without time zone,
    password character varying(255) NOT NULL,
    remember_token character varying(100),
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);

-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: -

CREATE SEQUENCE public.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;

-- Name: admin_audit_logs id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.admin_audit_logs ALTER COLUMN id SET DEFAULT nextval('public.admin_audit_logs_id_seq'::regclass);

-- Name: chat_messages id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.chat_messages ALTER COLUMN id SET DEFAULT nextval('public.chat_messages_id_seq'::regclass);

-- Name: firebase_images id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.firebase_images ALTER COLUMN id SET DEFAULT nextval('public.firebase_images_id_seq'::regclass);

-- Name: images id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.images ALTER COLUMN id SET DEFAULT nextval('public.images_id_seq'::regclass);

-- Name: invoice_lottery_numbers id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.invoice_lottery_numbers ALTER COLUMN id SET DEFAULT nextval('public.invoice_lottery_numbers_id_seq'::regclass);

-- Name: logs id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.logs ALTER COLUMN id SET DEFAULT nextval('public.logs_id_seq'::regclass);

-- Name: lotto_draws id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.lotto_draws ALTER COLUMN id SET DEFAULT nextval('public.lotto_draws_id_seq'::regclass);

-- Name: member_oauth id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.member_oauth ALTER COLUMN id SET DEFAULT nextval('public.member_oauth_id_seq'::regclass);

-- Name: members id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.members ALTER COLUMN id SET DEFAULT nextval('public.members_id_seq'::regclass);

-- Name: permissions id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.permissions ALTER COLUMN id SET DEFAULT nextval('public.permissions_id_seq'::regclass);

-- Name: roles id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.roles ALTER COLUMN id SET DEFAULT nextval('public.roles_id_seq'::regclass);

-- Name: stock_buyback_periods id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_buyback_periods ALTER COLUMN id SET DEFAULT nextval('public.stock_buyback_periods_id_seq'::regclass);

-- Name: stock_changes id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_changes ALTER COLUMN id SET DEFAULT nextval('public.stock_changes_id_seq'::regclass);

-- Name: stock_closing_prices id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_closing_prices ALTER COLUMN id SET DEFAULT nextval('public.stock_closing_prices_id_seq'::regclass);

-- Name: stock_day_all id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_day_all ALTER COLUMN id SET DEFAULT nextval('public.stock_day_all_id_seq'::regclass);

-- Name: torrents id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.torrents ALTER COLUMN id SET DEFAULT nextval('public.torrents_id_seq'::regclass);

-- Name: users id; Type: DEFAULT; Schema: public; Owner: -

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);

-- Name: admin_audit_logs admin_audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.admin_audit_logs
    ADD CONSTRAINT admin_audit_logs_pkey PRIMARY KEY (id);

-- Name: app_settings app_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.app_settings
    ADD CONSTRAINT app_settings_pkey PRIMARY KEY (key);

-- Name: blogs blogs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.blogs
    ADD CONSTRAINT blogs_pkey PRIMARY KEY (id);

-- Name: chat_messages chat_messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.chat_messages
    ADD CONSTRAINT chat_messages_pkey PRIMARY KEY (id);

-- Name: daily_visitor_stats daily_visitor_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.daily_visitor_stats
    ADD CONSTRAINT daily_visitor_stats_pkey PRIMARY KEY (date);

-- Name: firebase_images firebase_images_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.firebase_images
    ADD CONSTRAINT firebase_images_pkey PRIMARY KEY (id);

-- Name: hackmd_posts hackmd_posts_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.hackmd_posts
    ADD CONSTRAINT hackmd_posts_pkey PRIMARY KEY (id);

-- Name: hackmd_users hackmd_users_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.hackmd_users
    ADD CONSTRAINT hackmd_users_pkey PRIMARY KEY (user_path);

-- Name: images images_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_pkey PRIMARY KEY (id);

-- Name: images images_storage_key_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_storage_key_key UNIQUE (storage_key);

-- Name: invoice_lottery_numbers invoice_lottery_numbers_period_prize_tier_number_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.invoice_lottery_numbers
    ADD CONSTRAINT invoice_lottery_numbers_period_prize_tier_number_key UNIQUE (period, prize_tier, number);

-- Name: invoice_lottery_numbers invoice_lottery_numbers_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.invoice_lottery_numbers
    ADD CONSTRAINT invoice_lottery_numbers_pkey PRIMARY KEY (id);

-- Name: invoices invoices_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.invoices
    ADD CONSTRAINT invoices_pkey PRIMARY KEY (id);

-- Name: ledger_entries ledger_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.ledger_entries
    ADD CONSTRAINT ledger_entries_pkey PRIMARY KEY (id);

-- Name: logs logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.logs
    ADD CONSTRAINT logs_pkey PRIMARY KEY (id);

-- Name: lotto_draws lotto_draws_game_draw_date_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.lotto_draws
    ADD CONSTRAINT lotto_draws_game_draw_date_key UNIQUE (game, draw_date);

-- Name: lotto_draws lotto_draws_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.lotto_draws
    ADD CONSTRAINT lotto_draws_pkey PRIMARY KEY (id);

-- Name: lotto_tickets lotto_tickets_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.lotto_tickets
    ADD CONSTRAINT lotto_tickets_pkey PRIMARY KEY (id);

-- Name: member_oauth member_oauth_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.member_oauth
    ADD CONSTRAINT member_oauth_pkey PRIMARY KEY (id);

-- Name: member_oauth member_oauth_provider_provider_id_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.member_oauth
    ADD CONSTRAINT member_oauth_provider_provider_id_key UNIQUE (provider, provider_id);

-- Name: members members_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.members
    ADD CONSTRAINT members_pkey PRIMARY KEY (id);

-- Name: permissions permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.permissions
    ADD CONSTRAINT permissions_pkey PRIMARY KEY (id);

-- Name: permissions permissions_resource_action_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.permissions
    ADD CONSTRAINT permissions_resource_action_key UNIQUE (resource, action);

-- Name: portfolio portfolio_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.portfolio
    ADD CONSTRAINT portfolio_pkey PRIMARY KEY (id);

-- Name: role_permissions role_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.role_permissions
    ADD CONSTRAINT role_permissions_pkey PRIMARY KEY (role_id, permission_id);

-- Name: roles roles_name_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT roles_name_key UNIQUE (name);

-- Name: roles roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT roles_pkey PRIMARY KEY (id);

-- Name: stock_buyback_periods stock_buyback_periods_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_buyback_periods
    ADD CONSTRAINT stock_buyback_periods_pkey PRIMARY KEY (id);

-- Name: stock_buyback_periods stock_buyback_periods_stock_no_start_date_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_buyback_periods
    ADD CONSTRAINT stock_buyback_periods_stock_no_start_date_key UNIQUE (stock_no, start_date);

-- Name: stock_changes stock_changes_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_changes
    ADD CONSTRAINT stock_changes_pkey PRIMARY KEY (id);

-- Name: stock_changes stock_changes_stock_no_start_date_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_changes
    ADD CONSTRAINT stock_changes_stock_no_start_date_key UNIQUE (stock_no, start_date);

-- Name: stock_closing_prices stock_closing_prices_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_closing_prices
    ADD CONSTRAINT stock_closing_prices_pkey PRIMARY KEY (id);

-- Name: stock_day_all stock_day_all_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_day_all
    ADD CONSTRAINT stock_day_all_pkey PRIMARY KEY (id);

-- Name: stock_day_all stock_day_all_trade_date_stock_code_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_day_all
    ADD CONSTRAINT stock_day_all_trade_date_stock_code_key UNIQUE (trade_date, stock_code);

-- Name: stock_ex_rights_checked stock_ex_rights_checked_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_ex_rights_checked
    ADD CONSTRAINT stock_ex_rights_checked_pkey PRIMARY KEY (stock_no, from_date);

-- Name: stock_ex_rights stock_ex_rights_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.stock_ex_rights
    ADD CONSTRAINT stock_ex_rights_pkey PRIMARY KEY (stock_no, ex_date);

-- Name: torrents torrents_info_hash_key; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.torrents
    ADD CONSTRAINT torrents_info_hash_key UNIQUE (info_hash);

-- Name: torrents torrents_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.torrents
    ADD CONSTRAINT torrents_pkey PRIMARY KEY (id);

-- Name: user_roles user_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.user_roles
    ADD CONSTRAINT user_roles_pkey PRIMARY KEY (user_id, role_id);

-- Name: users users_email_unique; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_unique UNIQUE (email);

-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);

-- Name: idx_admin_audit_logs_created_at; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_admin_audit_logs_created_at ON public.admin_audit_logs USING btree (created_at DESC);

-- Name: idx_admin_audit_logs_user_email; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_admin_audit_logs_user_email ON public.admin_audit_logs USING btree (user_email);

-- Name: idx_chat_messages_user_name; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_chat_messages_user_name ON public.chat_messages USING btree (user_name);

-- Name: idx_invoices_member_number; Type: INDEX; Schema: public; Owner: -

CREATE UNIQUE INDEX idx_invoices_member_number ON public.invoices USING btree (member_id, invoice_number);

-- Name: idx_invoices_period_unchecked; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_invoices_period_unchecked ON public.invoices USING btree (period) WHERE (lottery_checked = false);

-- Name: idx_ledger_entries_member_date; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_ledger_entries_member_date ON public.ledger_entries USING btree (member_id, occurred_at DESC);

-- Name: idx_logs_created_at; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_logs_created_at ON public.logs USING btree (created_at DESC);

-- Name: idx_logs_level; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_logs_level ON public.logs USING btree (level);

-- Name: idx_lotto_tickets_member; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_lotto_tickets_member ON public.lotto_tickets USING btree (member_id, created_at DESC);

-- Name: idx_lotto_tickets_pending; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_lotto_tickets_pending ON public.lotto_tickets USING btree (game, draw_date) WHERE (checked = false);

-- Name: idx_stock_buyback_periods_end_date; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_stock_buyback_periods_end_date ON public.stock_buyback_periods USING btree (end_date);

-- Name: idx_stock_buyback_periods_start_date; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_stock_buyback_periods_start_date ON public.stock_buyback_periods USING btree (start_date);

-- Name: idx_stock_changes_status; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_stock_changes_status ON public.stock_changes USING btree (status);

-- Name: idx_stock_closing_prices_stock_date; Type: INDEX; Schema: public; Owner: -

CREATE UNIQUE INDEX idx_stock_closing_prices_stock_date ON public.stock_closing_prices USING btree (stock_no, date);

-- Name: idx_stock_day_all_stock_code_trade_date; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_stock_day_all_stock_code_trade_date ON public.stock_day_all USING btree (stock_code, trade_date DESC);

-- Name: idx_torrents_status; Type: INDEX; Schema: public; Owner: -

CREATE INDEX idx_torrents_status ON public.torrents USING btree (status);

-- Name: firebase_images update_firebase_images_timestamp; Type: TRIGGER; Schema: public; Owner: -

CREATE TRIGGER update_firebase_images_timestamp BEFORE UPDATE ON public.firebase_images FOR EACH ROW EXECUTE FUNCTION public.update_timestamp();

-- Name: invoices invoices_ledger_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.invoices
    ADD CONSTRAINT invoices_ledger_entry_id_fkey FOREIGN KEY (ledger_entry_id) REFERENCES public.ledger_entries(id) ON DELETE SET NULL;

-- Name: member_oauth member_oauth_member_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.member_oauth
    ADD CONSTRAINT member_oauth_member_id_fkey FOREIGN KEY (member_id) REFERENCES public.members(id) ON DELETE CASCADE;

-- Name: role_permissions role_permissions_permission_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.role_permissions
    ADD CONSTRAINT role_permissions_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public.permissions(id) ON DELETE CASCADE;

-- Name: role_permissions role_permissions_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.role_permissions
    ADD CONSTRAINT role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE CASCADE;

-- Name: user_roles user_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.user_roles
    ADD CONSTRAINT user_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE CASCADE;

-- Name: user_roles user_roles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.user_roles
    ADD CONSTRAINT user_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


-- ===== 種子資料（roles / permissions / role_permissions / app_settings）=====

-- Data for Name: app_settings; Type: TABLE DATA; Schema: public; Owner: -

INSERT INTO public.app_settings (key, value, description, category) VALUES ('hackmd_token', '', 'HackMD API Token', 'integration');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('google_client_id', '', 'Google OAuth Client ID', 'oauth');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('google_redirect_url', '', 'Google OAuth Redirect URL', 'oauth');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('github_client_id', '', 'GitHub OAuth Client ID', 'oauth');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('github_redirect_url', '', 'GitHub OAuth Redirect URL', 'oauth');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('line_client_id', '', 'LINE OAuth Client ID', 'oauth');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('line_redirect_url', '', 'LINE OAuth Redirect URL', 'oauth');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('upload_base_url', 'https://axum.kawa.homes/uploads', 'Upload Base URL（重啟後生效）', 'storage');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('cors_allowed_origins', 'https://kawa.homes', 'CORS 允許來源（逗號分隔多個）', 'cors');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('smtp_username', '', 'Gmail 寄件帳號', 'notification');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('smtp_password', '', 'Gmail App Password', 'notification');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('notify_email', '', '通知收件信箱（空白 = 同寄件帳號）', 'notification');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('default_color_mode', 'system', '全站深淺色預設（light / dark / system）', 'appearance');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('torrent_max_active', '2', '同時下載的 torrent 數量上限', 'torrent');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('torrent_retention_days', '7', 'completed / failed 後保留天數，逾期自動清除', 'torrent');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('torrent_max_total_size_gb', '20', 'torrent 目錄總容量上限（GB），超過拒收新任務', 'torrent');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('torrent_link_ttl_minutes', '180', '下載連結效期（分鐘）— 要涵蓋最大檔案在最慢線路的下載時間', 'torrent');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('site_theme', 'forest', '網站風格主題（forest / ocean / sky / sunset / sakura / grape / mono / auto）', 'appearance');
INSERT INTO public.app_settings (key, value, description, category) VALUES ('theme_rotation', '{"0":"forest","1":"ocean","2":"sky","3":"sunset","4":"sakura","5":"grape","6":"mono"}', '每日輪播主題對應表（星期 0=週日..6=週六 → 主題），site_theme=auto 時生效', 'appearance');

-- Data for Name: permissions; Type: TABLE DATA; Schema: public; Owner: -

INSERT INTO public.permissions (id, resource, action, description) VALUES (1, 'blog', 'read', '讀取文章');
INSERT INTO public.permissions (id, resource, action, description) VALUES (2, 'blog', 'create', '新增文章');
INSERT INTO public.permissions (id, resource, action, description) VALUES (3, 'blog', 'update', '編輯文章');
INSERT INTO public.permissions (id, resource, action, description) VALUES (4, 'blog', 'delete', '刪除文章');
INSERT INTO public.permissions (id, resource, action, description) VALUES (5, 'image', 'read', '讀取圖片');
INSERT INTO public.permissions (id, resource, action, description) VALUES (6, 'image', 'create', '上傳圖片');
INSERT INTO public.permissions (id, resource, action, description) VALUES (7, 'image', 'delete', '刪除圖片');
INSERT INTO public.permissions (id, resource, action, description) VALUES (8, 'note', 'read', '讀取筆記');
INSERT INTO public.permissions (id, resource, action, description) VALUES (9, 'note', 'create', '新增筆記');
INSERT INTO public.permissions (id, resource, action, description) VALUES (10, 'note', 'update', '編輯筆記');
INSERT INTO public.permissions (id, resource, action, description) VALUES (11, 'note', 'delete', '刪除筆記');
INSERT INTO public.permissions (id, resource, action, description) VALUES (12, 'stock', 'read', '讀取股票資料');
INSERT INTO public.permissions (id, resource, action, description) VALUES (13, 'stock', 'create', '新增股票資料');
INSERT INTO public.permissions (id, resource, action, description) VALUES (14, 'stock', 'update', '更新股票資料');
INSERT INTO public.permissions (id, resource, action, description) VALUES (15, 'stock', 'delete', '刪除股票資料');
INSERT INTO public.permissions (id, resource, action, description) VALUES (16, 'stock', 'manage', '管理股票任務');
INSERT INTO public.permissions (id, resource, action, description) VALUES (17, 'user', 'read', '讀取使用者');
INSERT INTO public.permissions (id, resource, action, description) VALUES (18, 'user', 'create', '新增使用者');
INSERT INTO public.permissions (id, resource, action, description) VALUES (19, 'user', 'update', '更新使用者');
INSERT INTO public.permissions (id, resource, action, description) VALUES (20, 'user', 'delete', '刪除使用者');
INSERT INTO public.permissions (id, resource, action, description) VALUES (21, 'role', 'read', '讀取角色');
INSERT INTO public.permissions (id, resource, action, description) VALUES (22, 'role', 'create', '新增角色');
INSERT INTO public.permissions (id, resource, action, description) VALUES (23, 'role', 'update', '更新角色');
INSERT INTO public.permissions (id, resource, action, description) VALUES (24, 'role', 'delete', '刪除角色');
INSERT INTO public.permissions (id, resource, action, description) VALUES (25, 'role', 'assign', '指派角色給使用者');
INSERT INTO public.permissions (id, resource, action, description) VALUES (26, 'member', 'read', '讀取會員資料');
INSERT INTO public.permissions (id, resource, action, description) VALUES (27, 'ws', 'read', '讀取 WebSocket 連線資訊');
INSERT INTO public.permissions (id, resource, action, description) VALUES (28, 'log', 'read', '讀取系統日誌');
INSERT INTO public.permissions (id, resource, action, description) VALUES (29, 'audit', 'read', '查詢 admin 操作紀錄');
INSERT INTO public.permissions (id, resource, action, description) VALUES (38, 'setting', 'read', NULL);
INSERT INTO public.permissions (id, resource, action, description) VALUES (39, 'setting', 'update', NULL);
INSERT INTO public.permissions (id, resource, action, description) VALUES (41, 'torrent', 'read', '查詢 torrent 任務與下載檔案');
INSERT INTO public.permissions (id, resource, action, description) VALUES (42, 'torrent', 'create', '新增 torrent 任務');
INSERT INTO public.permissions (id, resource, action, description) VALUES (43, 'torrent', 'delete', '刪除 torrent 任務與檔案');
INSERT INTO public.permissions (id, resource, action, description) VALUES (44, 'game', 'read', '查詢即時對局總覽');
INSERT INTO public.permissions (id, resource, action, description) VALUES (45, 'invoice_lottery', 'write', '手動輸入統一發票中獎號碼');
INSERT INTO public.permissions (id, resource, action, description) VALUES (46, 'stat', 'read', '查詢網站流量統計');

-- Data for Name: roles; Type: TABLE DATA; Schema: public; Owner: -

INSERT INTO public.roles (id, name, description, created_at) VALUES (1, 'guest', '訪客（未登入）', '2026-07-02 12:58:44.163058+00');
INSERT INTO public.roles (id, name, description, created_at) VALUES (2, 'member', '一般會員', '2026-07-02 12:58:44.163058+00');
INSERT INTO public.roles (id, name, description, created_at) VALUES (3, 'admin', '後台全功能', '2026-07-02 12:58:44.163058+00');
INSERT INTO public.roles (id, name, description, created_at) VALUES (4, 'super_admin', '含管理帳號/角色', '2026-07-02 12:58:44.163058+00');

-- Data for Name: role_permissions; Type: TABLE DATA; Schema: public; Owner: -

INSERT INTO public.role_permissions (role_id, permission_id) VALUES (1, 1);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (1, 5);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (1, 8);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (2, 1);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (2, 2);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (2, 5);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (2, 6);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (2, 8);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 1);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 2);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 3);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 4);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 5);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 6);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 7);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 8);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 9);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 10);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 11);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 12);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 13);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 14);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 15);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 16);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 17);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 18);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 19);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (3, 20);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 1);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 2);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 3);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 4);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 5);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 6);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 7);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 8);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 9);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 10);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 11);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 12);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 13);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 14);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 15);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 16);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 17);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 18);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 19);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 20);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 21);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 22);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 23);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 24);
INSERT INTO public.role_permissions (role_id, permission_id) VALUES (4, 25);

-- Name: permissions_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -

SELECT pg_catalog.setval('public.permissions_id_seq', 46, true);

-- Name: roles_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -

SELECT pg_catalog.setval('public.roles_id_seq', 4, true);


-- 對齊序列（種子以明確 id 插入）
SELECT setval('public.roles_id_seq', (SELECT MAX(id) FROM public.roles));
SELECT setval('public.permissions_id_seq', (SELECT MAX(id) FROM public.permissions));
