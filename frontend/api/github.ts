"use server";

import { parseCommitSubject, type ChangelogEntry } from "@/libs/changelog";

/**
 * /changelog 的資料來源：GitHub commits API（public repo，匿名 60 req/hr per IP）。
 *
 * 刻意不經後端：這頁只是唸 GitHub 的公開資料，走 Next Data Cache（revalidate 1h）
 * 就夠，不必為它在 backend 開 endpoint + Redis 快取。若哪天 instance 數變多、
 * repo 轉私有（要 token）或想在後台人工編修更新內容，再搬到後端。
 *
 * `GITHUB_REPO` 沒設時 fallback 本 repo；設成空字串 = 關閉這頁（商家 instance 用同一份
 * image，不該顯示 kawa 的 commit 紀錄）。
 */
const DEFAULT_REPO = "kawagami/api-server-axum";

/** 抓 100 筆再濾（chore/docs/deps 佔比高），顯示上限 40 筆 */
const FETCH_PER_PAGE = 100;
const MAX_ENTRIES = 40;

export async function getChangelogRepo(): Promise<string | null> {
    const value = process.env.GITHUB_REPO ?? DEFAULT_REPO;
    return value.trim() || null;
}

export async function getChangelog(): Promise<ChangelogEntry[]> {
    const repo = await getChangelogRepo();
    if (!repo) return [];

    // 抓不到就回空陣列讓頁面顯示空狀態 —— GitHub 掛掉不該讓整頁 500。
    // ISR 在熱狀態會保留舊快取並背景重試，只有容器剛啟動就失敗才真的空。
    try {
        const res = await fetch(
            `https://api.github.com/repos/${repo}/commits?per_page=${FETCH_PER_PAGE}`,
            {
                headers: {
                    Accept: "application/vnd.github+json",
                    "X-GitHub-Api-Version": "2022-11-28",
                    // GitHub 對沒帶 User-Agent 的請求回 403
                    "User-Agent": "kawa-homes-changelog",
                },
                next: { revalidate: 3600, tags: ["changelog"] },
            },
        );
        if (!res.ok) return [];

        const commits = (await res.json()) as GithubCommit[];
        if (!Array.isArray(commits)) return [];

        const entries: ChangelogEntry[] = [];
        for (const commit of commits) {
            const parsed = parseCommitSubject(commit.commit?.message ?? "");
            const date = commit.commit?.author?.date ?? commit.commit?.committer?.date;
            if (!parsed || !date || !commit.sha) continue;

            entries.push({
                ...parsed,
                sha: commit.sha.slice(0, 7),
                url: commit.html_url,
                date,
            });
            if (entries.length >= MAX_ENTRIES) break;
        }
        return entries;
    } catch {
        return [];
    }
}

/** GitHub commits API 回應中實際會用到的欄位 */
interface GithubCommit {
    sha: string;
    html_url: string;
    commit?: {
        message?: string;
        author?: { date?: string };
        committer?: { date?: string };
    };
}
