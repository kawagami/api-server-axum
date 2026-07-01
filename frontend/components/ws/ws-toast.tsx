"use client";

import { useWsNotification } from '@/hooks/useWsNotification';
import type { WsEventType } from '@/types';
import { X } from 'lucide-react';

interface ToastItemProps {
    data: unknown;
    onDismiss: () => void;
}

type ToastComponent = React.ComponentType<ToastItemProps>;

interface StockData {
    stock_name?: string;
    stock_no?: string;
}

interface BlogData {
    title?: string;
}

interface AdminMessageData {
    content?: string;
    from?: string;
}

function ToastShell({ bg, children, onDismiss }: { bg: string; children: React.ReactNode; onDismiss: () => void }) {
    return (
        <div className={`flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg text-white min-w-[260px] max-w-sm ${bg}`}>
            <div className="flex-1 text-sm">{children}</div>
            <button onClick={onDismiss} className="mt-0.5 hover:opacity-75">
                <X size={16} />
            </button>
        </div>
    );
}

function UserJoinedToast(_: ToastItemProps) { return null; }
function UserLeftToast(_: ToastItemProps) { return null; }

function BlogCreatedToast({ data, onDismiss }: ToastItemProps) {
    const blog = data as BlogData;
    return (
        <ToastShell bg="bg-primary-600 dark:bg-primary-700" onDismiss={onDismiss}>
            <p className="font-semibold">新文章發布</p>
            <p>{blog.title}</p>
        </ToastShell>
    );
}

function StockCompletedToast({ data, onDismiss }: ToastItemProps) {
    const s = data as StockData;
    return (
        <ToastShell bg="bg-green-600 dark:bg-green-700" onDismiss={onDismiss}>
            <p className="font-semibold">股票完成 ✓</p>
            <p>{s.stock_name}（{s.stock_no}）</p>
        </ToastShell>
    );
}

function StockFailedToast({ data, onDismiss }: ToastItemProps) {
    const s = data as StockData;
    return (
        <ToastShell bg="bg-red-600 dark:bg-red-700" onDismiss={onDismiss}>
            <p className="font-semibold">股票失敗 ✗</p>
            <p>股票代號：{s.stock_no}</p>
        </ToastShell>
    );
}

function AdminMessageToast({ data, onDismiss }: ToastItemProps) {
    const m = data as AdminMessageData;
    return (
        <ToastShell bg="bg-neutral-800 dark:bg-neutral-700" onDismiss={onDismiss}>
            <p className="font-semibold">管理員訊息</p>
            <p>{m.content}</p>
            {m.from && <p className="text-xs opacity-75">— {m.from}</p>}
        </ToastShell>
    );
}

const TOAST_MAP: Record<WsEventType, ToastComponent> = {
    user_joined: UserJoinedToast,
    user_left: UserLeftToast,
    blog_created: BlogCreatedToast,
    stock_completed: StockCompletedToast,
    stock_failed: StockFailedToast,
    admin_message: AdminMessageToast,
};

export default function WsToast() {
    const { notifications, dismiss } = useWsNotification();

    if (notifications.length === 0) return null;

    return (
        <div className="fixed bottom-6 right-6 z-50 flex flex-col gap-2">
            {notifications.map(({ id, type, data }) => {
                const Toast = TOAST_MAP[type];
                return <Toast key={id} data={data} onDismiss={() => dismiss(id)} />;
            })}
        </div>
    );
}
