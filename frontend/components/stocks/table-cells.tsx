import { TdHTMLAttributes, ThHTMLAttributes } from "react";

export function Td({ children, className = '', ...props }: TdHTMLAttributes<HTMLTableCellElement>) {
    return (
        <td className={`border border-neutral-300 dark:border-neutral-600 px-4 py-2 ${className}`} {...props}>
            {children}
        </td>
    );
}

export function Th({ children, className = '', ...props }: ThHTMLAttributes<HTMLTableCellElement>) {
    return (
        <th className={`border border-neutral-300 dark:border-neutral-600 px-4 py-2 dark:text-neutral-200 ${className}`} {...props}>
            {children}
        </th>
    );
}
