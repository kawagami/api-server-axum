export const METHOD_BADGE: Record<string, string> = {
    GET: "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400",
    POST: "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400",
    PUT: "bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400",
    PATCH: "bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400",
    DELETE: "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400",
};

export function httpStatusBadgeClass(code: number): string {
    if (code >= 500) return "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400";
    if (code >= 400) return "bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400";
    if (code >= 300) return "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400";
    return "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400";
}

export const LEVEL_BADGE: Record<string, string> = {
    ERROR: "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400",
    WARN: "bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400",
    INFO: "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400",
};

export const TORRENT_STATUS_BADGE: Record<string, string> = {
    pending: "bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400",
    downloading: "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400",
    completed: "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400",
    failed: "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400",
};

export const LEVEL_ROW_BG: Record<string, string> = {
    ERROR: "bg-red-50 dark:bg-red-900/10 hover:bg-red-100/70 dark:hover:bg-red-900/20",
    WARN: "bg-yellow-50 dark:bg-yellow-900/10 hover:bg-yellow-100/70 dark:hover:bg-yellow-900/20",
    INFO: "hover:bg-neutral-50 dark:hover:bg-neutral-800/50",
};
