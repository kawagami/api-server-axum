const cellBorder = "border border-neutral-300 dark:border-neutral-700 px-4 py-2";

export function AdminTable({ className = "", ...props }: React.ComponentProps<"table">) {
    return <table className={`w-full border-collapse border border-neutral-200 dark:border-neutral-700 ${className}`} {...props} />;
}

export function AdminHeadRow({ className = "", ...props }: React.ComponentProps<"tr">) {
    return <tr className={`bg-neutral-100 dark:bg-neutral-800 ${className}`} {...props} />;
}

export function AdminRow({ className = "", ...props }: React.ComponentProps<"tr">) {
    return <tr className={`hover:bg-neutral-50 dark:hover:bg-neutral-800 ${className}`} {...props} />;
}

export function AdminTh({ className = "", ...props }: React.ComponentProps<"th">) {
    return <th className={`${cellBorder} text-left text-neutral-700 dark:text-neutral-300 ${className}`} {...props} />;
}

export function AdminTd({ className = "", ...props }: React.ComponentProps<"td">) {
    return <td className={`${cellBorder} text-neutral-900 dark:text-neutral-100 ${className}`} {...props} />;
}
