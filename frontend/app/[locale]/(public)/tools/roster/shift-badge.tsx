interface ShiftStyle {
    container: string;
    dot: string;
}

const shiftStyles: Record<string, ShiftStyle> = {
    "早班": { container: "bg-primary-100 text-primary-700 dark:bg-primary-900/40 dark:text-primary-300 border-primary-200 dark:border-primary-800", dot: "bg-primary-500" },
    "晚班": { container: "bg-primary-100 text-primary-700 dark:bg-primary-900/40 dark:text-primary-300 border-primary-200 dark:border-primary-800", dot: "bg-primary-500" },
    "休": { container: "bg-neutral-100 text-neutral-400 dark:bg-neutral-800/60 dark:text-neutral-500 border-neutral-200 dark:border-neutral-700", dot: "bg-neutral-400" },
    "default": { container: "bg-orange-100 text-orange-700 dark:bg-orange-900/40 dark:text-orange-300 border-orange-200 dark:border-orange-800", dot: "bg-orange-500" },
};

export default function ShiftBadge({ type }: { type: string }) {
    const currentStyle = shiftStyles[type] ?? shiftStyles["default"];

    return (
        <span className={`inline-flex items-center justify-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border transition-all duration-200 shadow-sm ${currentStyle.container}`}>
            <span className={`w-1.5 h-1.5 rounded-full ${currentStyle.dot}`} />
            {type}
        </span>
    );
}
