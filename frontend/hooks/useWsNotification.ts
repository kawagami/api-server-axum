"use client";

import { useState } from 'react';
import { useWsSubscribe } from '@/hooks/useWsSubscribe';
import type { WsNotification, WsEventType } from '@/types';

export function useWsNotification() {
    const [notifications, setNotifications] = useState<WsNotification[]>([]);

    const addNotification = (type: WsEventType) => (data: unknown) => {
        const id = Date.now() + Math.random();
        setNotifications(prev => [...prev, { id, type, data }]);
        setTimeout(() => {
            setNotifications(prev => prev.filter(n => n.id !== id));
        }, 5000);
    };

    useWsSubscribe('stock_completed', addNotification('stock_completed'));
    useWsSubscribe('stock_failed', addNotification('stock_failed'));
    useWsSubscribe('blog_created', addNotification('blog_created'));
    useWsSubscribe('user_joined', addNotification('user_joined'));
    useWsSubscribe('user_left', addNotification('user_left'));
    useWsSubscribe('admin_message', addNotification('admin_message'));

    const dismiss = (id: number) =>
        setNotifications(prev => prev.filter(n => n.id !== id));

    return { notifications, dismiss };
}
