// Torrent
export type TorrentStatus = 'pending' | 'downloading' | 'completed' | 'failed';

export interface TorrentFile {
  index: number;
  path: string;
  size: number;
}

// 即時進度（不存 DB，僅進行中任務的詳情 API 與 WS 推播帶）
export interface TorrentLive {
  progress: number;
  progress_bytes: number;
  total_bytes: number;
  down_speed: string;
  peers: number;
}

export interface Torrent {
  id: number;
  info_hash: string;
  magnet_uri: string;
  name: string | null;
  status: TorrentStatus;
  total_size: number | null;
  files: TorrentFile[] | null;
  error: string | null;
  created_by: string;
  created_at: string;
  completed_at: string | null;
  live?: TorrentLive;
}

export interface TorrentPaginatedResponse {
  data: Torrent[];
  total: number;
}

export interface TorrentDownloadLink {
  file_index: number;
  path: string;
  size: number;
  url: string;
  expires_at: string;
}

export interface TorrentStorage {
  disk: { total_bytes: number; available_bytes: number };
  torrent: { used_bytes: number; max_bytes: number };
}

export interface TorrentProgressEvent extends TorrentLive {
  id: number;
  name: string;
}

export interface TorrentCompletedEvent {
  id: number;
  name: string;
  total_size: number;
}

export interface TorrentFailedEvent {
  id: number;
  name: string;
  reason: string;
}
