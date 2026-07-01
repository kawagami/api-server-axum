"use client";

import { useState, useRef, useEffect } from 'react';
import Image from 'next/image';
import { useRouter } from 'next/navigation';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import { Loader2, Bold, Italic, Code, Link2, Heading2, Quote, List } from 'lucide-react';
import 'highlight.js/styles/github-dark.css';
import { putBlog } from '@/api/blogs';
import { uploadImages } from '@/api/images';
import { useMarkdownTextarea } from '@/hooks/useMarkdownTextarea';
import type { Blog, Toc } from '@/types';

function extractTocs(markdown: string): Toc[] {
    return markdown.match(/^#{1,6}\s+(.+)$/gm)?.map((h, index) => ({
        id: String(index),
        level: h.match(/^#+/)![0].length,
        text: h.replace(/^#{1,6}\s+/, ''),
    })) || [];
}

interface Props {
    id: string;
    blog: Blog;
    allTags: string[];
}

export default function BlogComponent({ id, blog, allTags }: Props) {
    const router = useRouter();
    const draftKey = `blog-draft:${id}`;
    const [markdown, setMarkdown] = useState(blog.markdown || '');
    const [tags, setTags] = useState<string[]>(blog.tags || []);
    const [isSaving, setIsSaving] = useState(false);
    const [isUploading, setIsUploading] = useState(false);
    const [saveError, setSaveError] = useState<string | null>(null);
    const [uploadError, setUploadError] = useState<string | null>(null);
    const [showTagModal, setShowTagModal] = useState(false);
    const [newTag, setNewTag] = useState('');
    const [draftRestored, setDraftRestored] = useState(false);
    const fileInputRef = useRef<HTMLInputElement>(null);
    const previewRef = useRef<HTMLDivElement>(null);
    const { ref: textareaRef, handlers: editorHandlers, insert: insertAtCursor, wrap, prefix } = useMarkdownTextarea(
        markdown,
        setMarkdown,
        { onImageUpload: handleImageUpload },
    );

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

    // Close the tag modal on Escape.
    useEffect(() => {
        if (!showTagModal) return;
        const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') setShowTagModal(false); };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, [showTagModal]);

    const handleSave = async () => {
        setIsSaving(true);
        setSaveError(null);
        try {
            const tocs = extractTocs(markdown);
            await putBlog(id, { markdown, tags, tocs });
            localStorage.removeItem(draftKey);
            router.push('/admin/blogs');
        } catch (err) {
            if ((err as { digest?: string }).digest?.startsWith('NEXT_REDIRECT')) throw err;
            setSaveError('存檔失敗，請再試一次');
            setIsSaving(false);
        }
    };

    async function handleImageUpload(files: File | FileList | null) {
        if (!files || isUploading) return;
        const fileArray = files instanceof File ? [files] : Array.from(files);
        if (!fileArray.length) return;
        setIsUploading(true);
        setUploadError(null);
        const formData = new FormData();
        fileArray.forEach(f => formData.append('file', f));
        try {
            const data = await uploadImages(formData);
            insertAtCursor(data.map(d => `![image](${d.url})`).join('\n') + '\n');
        } catch (err) {
            setUploadError('圖片上傳失敗，請再試一次');
        } finally {
            setIsUploading(false);
        }
    }

    const handleAddTag = () => {
        if (newTag.trim() && !tags.includes(newTag.trim())) {
            setTags([...tags, newTag.trim()]);
            setNewTag('');
        }
    };

    // Sync preview scroll position to the editor's.
    const handleEditorScroll = () => {
        const ta = textareaRef.current;
        const pv = previewRef.current;
        if (!ta || !pv) return;
        const ratio = ta.scrollTop / (ta.scrollHeight - ta.clientHeight || 1);
        pv.scrollTop = ratio * (pv.scrollHeight - pv.clientHeight);
    };

    const toolBtn = "p-2 rounded text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors";

    return (
        <>
            <div className="lg:h-[calc(100svh-180px)] w-full flex flex-col">
                {draftRestored && (
                    <p className="text-primary-600 dark:text-primary-400 text-sm text-center mt-2">已從本機草稿復原未存檔內容</p>
                )}
                {saveError && <p className="text-red-500 text-sm text-center mt-2">{saveError}</p>}
                {uploadError && <p className="text-red-500 text-sm text-center mt-2">{uploadError}</p>}
                <div className="flex flex-wrap justify-evenly items-center m-4 gap-2">
                    <button
                        onClick={handleSave}
                        disabled={isSaving}
                        className={`px-6 py-2 font-semibold rounded-lg shadow-md transition-colors ${isSaving
                            ? 'bg-neutral-400 text-neutral-700 cursor-not-allowed'
                            : 'bg-primary-600 text-white hover:bg-primary-700'}`}
                    >
                        {isSaving ? (
                            <span className="flex items-center gap-1"><Loader2 className="w-4 h-4 animate-spin" />存檔中...</span>
                        ) : '存檔'}
                    </button>
                    <button
                        onClick={() => setShowTagModal(true)}
                        className="px-6 py-2 font-semibold bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
                    >
                        編輯類型
                    </button>
                    <button
                        onClick={() => fileInputRef.current?.click()}
                        disabled={isSaving || isUploading}
                        className="px-6 py-2 font-semibold bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:bg-neutral-400 disabled:cursor-not-allowed flex items-center gap-2 transition-colors"
                    >
                        {isUploading && <Loader2 className="w-4 h-4 animate-spin" />}
                        {isUploading ? '上傳中...' : '上傳圖片'}
                    </button>
                    <input
                        ref={fileInputRef}
                        type="file"
                        accept="image/*"
                        multiple
                        className="hidden"
                        onChange={(e) => handleImageUpload(e.target.files)}
                    />
                </div>

                <div className="flex flex-wrap items-center gap-1 px-4 mb-2 border-b border-neutral-200 dark:border-neutral-700 pb-2">
                    <button type="button" title="粗體" className={toolBtn} onClick={() => wrap('**', '**', '粗體')}><Bold className="w-4 h-4" /></button>
                    <button type="button" title="斜體" className={toolBtn} onClick={() => wrap('*', '*', '斜體')}><Italic className="w-4 h-4" /></button>
                    <button type="button" title="行內碼" className={toolBtn} onClick={() => wrap('`', '`', 'code')}><Code className="w-4 h-4" /></button>
                    <button type="button" title="連結" className={toolBtn} onClick={() => wrap('[', '](url)', '文字')}><Link2 className="w-4 h-4" /></button>
                    <button type="button" title="標題" className={toolBtn} onClick={() => prefix('## ')}><Heading2 className="w-4 h-4" /></button>
                    <button type="button" title="引用" className={toolBtn} onClick={() => prefix('> ')}><Quote className="w-4 h-4" /></button>
                    <button type="button" title="清單" className={toolBtn} onClick={() => prefix('- ')}><List className="w-4 h-4" /></button>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 px-4 flex-1 min-h-0">
                    <div className="relative h-full min-h-[300px]">
                        <textarea
                            ref={textareaRef}
                            value={markdown}
                            onChange={(e) => setMarkdown(e.target.value)}
                            onScroll={handleEditorScroll}
                            {...editorHandlers}
                            className="w-full h-full p-4 rounded border border-neutral-300 font-mono resize-none dark:bg-neutral-800 dark:text-white dark:border-neutral-600"
                            placeholder="輸入 Markdown 內容..."
                        />
                    </div>
                    <div ref={previewRef} className="p-4 h-full min-h-[300px] overflow-auto border border-neutral-300 bg-white dark:bg-neutral-800 dark:text-white rounded prose max-w-none dark:prose-invert">
                        <ReactMarkdown
                            remarkPlugins={[remarkGfm]}
                            rehypePlugins={[rehypeHighlight]}
                            urlTransform={(url) => url.startsWith('blob:') || url.startsWith('https://') || url.startsWith('http://') || url.startsWith('/') ? url : ''}
                            components={{
                                img: ({ src, alt }) => (
                                    <Image
                                        src={typeof src === 'string' ? src : ''}
                                        alt={alt || ''}
                                        width={800}
                                        height={600}
                                        style={{ width: 'auto', height: 'auto', maxWidth: '100%' }}
                                    />
                                )
                            }}
                        >
                            {markdown}
                        </ReactMarkdown>
                    </div>
                </div>
            </div>

            {showTagModal && (
                <div
                    className="fixed inset-0 z-10 flex items-center justify-center bg-black/50 p-4"
                    onClick={() => setShowTagModal(false)}
                >
                    <div
                        className="bg-white dark:bg-neutral-800 p-6 rounded-lg shadow-lg w-full max-w-md max-h-[80vh] overflow-auto"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <h2 className="text-lg font-semibold mb-4">編輯類型</h2>
                        <input
                            type="text"
                            value={newTag}
                            onChange={(e) => setNewTag(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleAddTag()}
                            className="w-full p-2 mb-4 border rounded dark:bg-neutral-700"
                            placeholder="輸入新類型..."
                        />
                        <button
                            onClick={handleAddTag}
                            className="w-full mb-4 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
                        >
                            新增類型
                        </button>
                        <ul className="space-y-2">
                            {tags.map((tag, index) => (
                                <li key={index} className="flex justify-between items-center">
                                    <span>{tag}</span>
                                    <button
                                        onClick={() => setTags(tags.filter((_, i) => i !== index))}
                                        className="text-red-600 hover:underline"
                                    >
                                        移除
                                    </button>
                                </li>
                            ))}
                        </ul>
                        <div className="mt-4">
                            <h3 className="text-md font-medium mb-2">所有類型</h3>
                            <ul className="space-y-2 max-h-40 overflow-auto border-t pt-2">
                                {allTags.map((tag, index) => (
                                    <li key={index} className="flex justify-between items-center">
                                        <span>{tag}</span>
                                        <button
                                            onClick={() => { if (!tags.includes(tag)) setTags([...tags, tag]); }}
                                            className="text-primary-600 hover:underline"
                                        >
                                            新增
                                        </button>
                                    </li>
                                ))}
                            </ul>
                        </div>
                        <button
                            onClick={() => setShowTagModal(false)}
                            className="mt-4 w-full px-4 py-2 bg-neutral-600 text-white rounded-lg hover:bg-neutral-700 transition-colors"
                        >
                            關閉
                        </button>
                    </div>
                </div>
            )}
        </>
    );
}
