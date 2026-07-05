"use client";

import { useState, useEffect } from 'react';

interface Props {
    tags: string[];
    allTags: string[];
    onTagsChange: (tags: string[]) => void;
    onClose: () => void;
}

export default function TagEditorModal({ tags, allTags, onTagsChange, onClose }: Props) {
    const [newTag, setNewTag] = useState('');

    // Close the tag modal on Escape.
    useEffect(() => {
        const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, [onClose]);

    const handleAddTag = () => {
        if (newTag.trim() && !tags.includes(newTag.trim())) {
            onTagsChange([...tags, newTag.trim()]);
            setNewTag('');
        }
    };

    return (
        <div
            className="fixed inset-0 z-10 flex items-center justify-center bg-black/50 p-4"
            onClick={onClose}
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
                                onClick={() => onTagsChange(tags.filter((_, i) => i !== index))}
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
                                    onClick={() => { if (!tags.includes(tag)) onTagsChange([...tags, tag]); }}
                                    className="text-primary-600 hover:underline"
                                >
                                    新增
                                </button>
                            </li>
                        ))}
                    </ul>
                </div>
                <button
                    onClick={onClose}
                    className="mt-4 w-full px-4 py-2 bg-neutral-600 text-white rounded-lg hover:bg-neutral-700 transition-colors"
                >
                    關閉
                </button>
            </div>
        </div>
    );
}
