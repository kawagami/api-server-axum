// 後端分頁回應的共用形狀（Rust 側是 structs/pagination.rs 的 `Paginated<T>`）
//
// `total` 是套用篩選後、未套 limit/offset 的總筆數。
// 曾經有 5 處各自 inline 的 `{ data: T[]; total: number }`（gov-tender / torrent /
// stock / blog-comment / blog），已全數改成 alias 或直接用這支。
// 分頁回應**一律**用這個，不要再 inline 一份。
export interface PaginatedResponse<T> {
    data: T[];
    total: number;
}
