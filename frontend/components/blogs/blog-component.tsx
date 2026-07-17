"use client";

import { useState, useRef } from 'react';
import Image from 'next/image';
import { useRouter } from 'next/navigation';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import { Loader2, Bold, Italic, Code, Link2, Heading2, Quote, List, Plus, X } from 'lucide-react';
import 'highlight.js/styles/github-dark.css';
import { putBlog } from '@/api/blogs';
import { uploadImages } from '@/api/images';
import { validateFileSizes, uploadErrorMessage, uploadProgressLabel, withUploadTimeout, type UploadProgress } from '@/libs/upload-limits';
import { compressImages } from '@/libs/client-image';
import { useMarkdownTextarea } from '@/hooks/useMarkdownTextarea';
import { useBlogDraft } from './useBlogDraft';
import TagEditorModal from './tag-editor-modal';
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
    const { markdown, setMarkdown, tags, setTags, draftRestored, clearDraft } = useBlogDraft(id, blog);
    const [isSaving, setIsSaving] = useState(false);
    const [isUploading, setIsUploading] = useState(false);
    const [saveError, setSaveError] = useState<string | null>(null);
    const [uploadProgress, setUploadProgress] = useState<UploadProgress | null>(null);
    const [uploadError, setUploadError] = useState<string | null>(null);
    const [showTagModal, setShowTagModal] = useState(false);
    const fileInputRef = useRef<HTMLInputElement>(null);
    const previewRef = useRef<HTMLDivElement>(null);
    const { ref: textareaRef, handlers: editorHandlers, insert: insertAtCursor, wrap, prefix } = useMarkdownTextarea(
        markdown,
        setMarkdown,
        { onImageUpload: handleImageUpload },
    );

    const handleSave = async () => {
        setIsSaving(true);
        setSaveError(null);
        try {
            const tocs = extractTocs(markdown);
            await putBlog(id, { markdown, tags, tocs });
            clearDraft();
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
        try {
            const compressed = await compressImages(fileArray, (current, total) =>
                setUploadProgress({ phase: 'compress', current, total }));
            // 大小檢查在壓縮後做:多數超限原圖壓完就過了,擋不住的只剩大 GIF / 解不開的檔
            const sizeError = validateFileSizes(compressed);
            if (sizeError) {
                setUploadError(sizeError);
                return;
            }
            // 一張一請求逐張上傳(part 數最少避 WAF 誤殺、後端單張處理遠低於 30 秒逾時),每張完成即插入 markdown
            for (let i = 0; i < compressed.length; i++) {
                setUploadProgress({ phase: 'upload', current: i + 1, total: compressed.length });
                const formData = new FormData();
                formData.append('file', compressed[i]);
                const data = await withUploadTimeout(uploadImages(formData));
                insertAtCursor(data.map(d => `![image](${d.url})`).join('\n') + '\n');
            }
        } catch (err) {
            setUploadError(uploadErrorMessage(err));
        } finally {
            setIsUploading(false);
            setUploadProgress(null);
        }
    }

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
                        onClick={() => fileInputRef.current?.click()}
                        disabled={isSaving || isUploading}
                        className="px-6 py-2 font-semibold bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:bg-neutral-400 disabled:cursor-not-allowed flex items-center gap-2 transition-colors"
                    >
                        {isUploading && <Loader2 className="w-4 h-4 animate-spin" />}
                        {isUploading ? uploadProgressLabel(uploadProgress) : '上傳圖片'}
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

                <div className="flex flex-wrap items-center gap-2 px-4 mb-2">
                    {tags.map((tag) => (
                        <span
                            key={tag}
                            className="flex items-center gap-1 bg-primary-50 dark:bg-primary-900/40 border border-primary-200 dark:border-primary-800 rounded-lg pl-2.5 pr-1 py-0.5 text-sm text-neutral-700 dark:text-neutral-200"
                        >
                            {tag}
                            <button
                                onClick={() => setTags(tags.filter((t) => t !== tag))}
                                aria-label={`移除 ${tag}`}
                                className="p-1.5 rounded text-neutral-400 hover:text-red-600 transition-colors"
                            >
                                <X size={14} />
                            </button>
                        </span>
                    ))}
                    <button
                        onClick={() => setShowTagModal(true)}
                        className="flex items-center gap-1 px-2.5 py-0.5 text-sm rounded-lg border border-dashed border-neutral-300 dark:border-neutral-600 text-neutral-500 dark:text-neutral-400 hover:border-primary-400 hover:text-primary-600 dark:hover:text-primary-300 transition-colors"
                    >
                        <Plus size={14} /> 編輯 Tag
                    </button>
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
                <TagEditorModal
                    tags={tags}
                    allTags={allTags}
                    onTagsChange={setTags}
                    onClose={() => setShowTagModal(false)}
                />
            )}
        </>
    );
}
