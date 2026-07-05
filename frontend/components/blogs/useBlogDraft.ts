"use client";

import { useState, useEffect } from 'react';
import type { Blog } from '@/types';

// localStorage 草稿：掛載時 restore、dirty 時 debounce 自動存、離頁前警告未存檔
export function useBlogDraft(id: string, blog: Blog) {
    const draftKey = `blog-draft:${id}`;
    const [markdown, setMarkdown] = useState(blog.markdown || '');
    const [tags, setTags] = useState<string[]>(blog.tags || []);
    const [draftRestored, setDraftRestored] = useState(false);

    const dirty = markdown !== (blog.markdown || '')
        || JSON.stringify(tags) !== JSON.stringify(blog.tags || []);

    // Restore a local draft on mount if it diverges from the saved blog.
    useEffect(() => {
        const raw = localStorage.getItem(draftKey);
        if (!raw) return;
        try {
            const d = JSON.parse(raw);
            if (typeof d.markdown === 'string' && d.markdown !== (blog.markdown || '')) {
                /* eslint-disable react-hooks/set-state-in-effect */
                setMarkdown(d.markdown);
                setTags(Array.isArray(d.tags) ? d.tags : (blog.tags || []));
                setDraftRestored(true);
                /* eslint-enable react-hooks/set-state-in-effect */
            }
        } catch { /* ignore corrupt draft */ }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    // Persist a debounced draft while dirty; clear it once clean.
    useEffect(() => {
        if (!dirty) {
            localStorage.removeItem(draftKey);
            return;
        }
        const t = setTimeout(() => {
            localStorage.setItem(draftKey, JSON.stringify({ markdown, tags }));
        }, 500);
        return () => clearTimeout(t);
    }, [markdown, tags, dirty, draftKey]);

    // Warn before leaving with unsaved changes.
    useEffect(() => {
        if (!dirty) return;
        const handler = (e: BeforeUnloadEvent) => {
            e.preventDefault();
            e.returnValue = '';
        };
        window.addEventListener('beforeunload', handler);
        return () => window.removeEventListener('beforeunload', handler);
    }, [dirty]);

    const clearDraft = () => localStorage.removeItem(draftKey);

    return { markdown, setMarkdown, tags, setTags, draftRestored, clearDraft };
}
