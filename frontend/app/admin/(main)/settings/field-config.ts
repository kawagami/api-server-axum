// 通用設定表單的 per-key 欄位型別。未登記的 key 一律當純文字 input;
// 新增列舉/數字/秘密類設定時在這裡補一行即可
export type FieldConfig =
    | { kind: "enum"; options: string[] }
    | { kind: "number"; min?: number; max?: number }
    | { kind: "secret" };

export const FIELD_CONFIGS: Record<string, FieldConfig> = {
    default_color_mode: { kind: "enum", options: ["light", "dark", "system"] },
    smtp_password: { kind: "secret" },
    torrent_max_active: { kind: "number", min: 1 },
    torrent_retention_days: { kind: "number", min: 1 },
    torrent_max_total_size_gb: { kind: "number", min: 1 },
    torrent_link_ttl_minutes: { kind: "number", min: 1 },
    image_webp_quality: { kind: "number", min: 1, max: 100 },
    image_client_compress: { kind: "enum", options: ["true", "false"] },
    image_client_quality: { kind: "number", min: 1, max: 100 },
    image_client_max_edge: { kind: "number", min: 64, max: 16383 },
};

// category slug → 中文標題(未登記的 fallback 顯示原字串)
export const CATEGORY_LABELS: Record<string, string> = {
    appearance: "外觀",
    oauth: "OAuth",
    notification: "通知",
    storage: "儲存",
    cors: "CORS",
    torrent: "Torrent",
    gov_tender: "標案追蹤",
    user: "管理員",
    integration: "整合",
};
