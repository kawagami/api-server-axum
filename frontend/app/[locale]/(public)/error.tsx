"use client";

export default function GlobalError({ error, reset }: { error: Error; reset: () => void }) {
    return (
        <div className="flex flex-col items-center justify-center h-[calc(100svh-120px)] gap-4 text-center px-4">
            <h1 className="text-4xl font-bold text-neutral-700 dark:text-neutral-200">發生錯誤</h1>
            <p className="text-neutral-500 dark:text-neutral-400">伺服器回應異常，請稍後再試。</p>
            {process.env.NODE_ENV === 'development' && (
                <pre className="text-xs text-red-400 max-w-sm overflow-auto">{error.message}</pre>
            )}
            <button
                onClick={reset}
                className="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
            >
                重試
            </button>
        </div>
    );
}
