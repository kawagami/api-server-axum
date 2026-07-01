"use client";

import { useEffect, useRef } from 'react';
import { useWsContext } from '@/libs/ws-context';

export function useWsSubscribe(type: string, fn: (data: unknown) => void) {
    const { subscribe, unsubscribe } = useWsContext();
    const fnRef = useRef(fn);

    useEffect(() => {
        fnRef.current = fn;
    }, [fn]);

    useEffect(() => {
        const handler = (data: unknown) => fnRef.current(data);
        subscribe(type, handler);
        return () => unsubscribe(type, handler);
    }, [type, subscribe, unsubscribe]);
}
