// 系統指標（admin）：後端排程定時採樣 CPU / 記憶體 / 磁碟 / load，時間由舊到新排序
export interface SystemMetric {
  id: number;
  cpu_pct: number; // 0~100
  mem_used_mb: number;
  mem_total_mb: number;
  disk_used_mb: number;
  disk_total_mb: number;
  load1: number;
  load5: number;
  load15: number;
  backend_rss_mb: number; // backend 行程自身常駐記憶體（MB），與整機 mem 分開
  created_at: string; // ISO 時間字串（UTC）
}
