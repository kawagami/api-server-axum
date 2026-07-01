"use client";

import { useState, useTransition } from 'react';
import { Trash2, ChevronDown, ChevronRight, Plus, Loader2 } from 'lucide-react';
import { createRole, deleteRole, setRolePermissions } from '@/app/admin/(main)/roles/actions';
import type { Role, Permission } from '@/types';

interface Props {
    initialRoles: Role[];
    allPermissions: Permission[];
}

export default function RolesManager({ initialRoles, allPermissions }: Props) {
    const [roles, setRoles] = useState(initialRoles);
    const [expandedId, setExpandedId] = useState<number | null>(null);
    const [isPending, startTransition] = useTransition();
    const [createError, setCreateError] = useState<string | null>(null);

    function toggleExpand(id: number) {
        setExpandedId(prev => (prev === id ? null : id));
    }

    async function handleCreate(formData: FormData) {
        setCreateError(null);
        try {
            await createRole(formData);
        } catch {
            setCreateError('建立失敗');
        }
    }

    function handleDelete(id: number) {
        startTransition(async () => {
            await deleteRole(id);
            setRoles(prev => prev.filter(r => r.id !== id));
            if (expandedId === id) setExpandedId(null);
        });
    }

    function handlePermissionToggle(role: Role, permission: Permission) {
        const current = role.permissions ?? [];
        const has = current.some(p => p.id === permission.id);
        const next = has
            ? current.filter(p => p.id !== permission.id)
            : [...current, permission];

        startTransition(async () => {
            await setRolePermissions(role.id, next.map(p => p.id));
            setRoles(prev =>
                prev.map(r => (r.id === role.id ? { ...r, permissions: next } : r))
            );
        });
    }

    return (
        <div className="max-w-3xl mx-auto space-y-6">
            <h1 className="text-2xl font-bold text-neutral-800 dark:text-white">角色管理</h1>

            {/* Create form */}
            <form action={handleCreate} className="flex gap-2">
                <input
                    name="name"
                    required
                    placeholder="角色名稱"
                    className="flex-1 px-3 py-2 border rounded-md text-sm dark:bg-neutral-800 dark:border-neutral-600 dark:text-white"
                />
                <input
                    name="description"
                    placeholder="說明（選填）"
                    className="flex-1 px-3 py-2 border rounded-md text-sm dark:bg-neutral-800 dark:border-neutral-600 dark:text-white"
                />
                <button
                    type="submit"
                    className="flex items-center gap-1 px-4 py-2 text-sm text-white bg-primary-600 rounded-md hover:bg-primary-700"
                >
                    <Plus size={14} />
                    新增
                </button>
            </form>
            {createError && <p className="text-sm text-red-500">{createError}</p>}

            {/* Role list */}
            <div className="space-y-2">
                {roles.map(role => (
                    <div key={role.id} className="border rounded-lg dark:border-neutral-700">
                        <div className="flex items-center justify-between px-4 py-3">
                            <button
                                onClick={() => toggleExpand(role.id)}
                                className="flex items-center gap-2 text-left flex-1"
                            >
                                {expandedId === role.id ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                                <span className="font-medium dark:text-white">{role.name}</span>
                                {role.description && (
                                    <span className="text-sm text-neutral-500 dark:text-neutral-400">— {role.description}</span>
                                )}
                            </button>
                            <button
                                onClick={() => handleDelete(role.id)}
                                disabled={isPending}
                                className="p-1 text-red-500 hover:text-red-700 dark:text-red-400 disabled:opacity-50"
                            >
                                {isPending ? <Loader2 size={16} className="animate-spin" /> : <Trash2 size={16} />}
                            </button>
                        </div>

                        {expandedId === role.id && (
                            <div className="px-4 pb-4 border-t dark:border-neutral-700">
                                <p className="text-xs text-neutral-500 dark:text-neutral-400 mt-3 mb-2">權限設定（點擊切換）</p>
                                <div className="flex flex-wrap gap-2">
                                    {allPermissions.map(perm => {
                                        const active = role.permissions?.some(p => p.id === perm.id) ?? false;
                                        return (
                                            <button
                                                key={perm.id}
                                                onClick={() => handlePermissionToggle(role, perm)}
                                                disabled={isPending}
                                                className={`px-2 py-1 text-xs rounded-full border transition-colors ${
                                                    active
                                                        ? 'bg-primary-100 border-primary-400 text-primary-700 dark:bg-primary-900 dark:border-primary-500 dark:text-primary-300'
                                                        : 'bg-neutral-100 border-neutral-300 text-neutral-600 dark:bg-neutral-800 dark:border-neutral-600 dark:text-neutral-400'
                                                }`}
                                            >
                                                {perm.resource}:{perm.action}
                                            </button>
                                        );
                                    })}
                                </div>
                            </div>
                        )}
                    </div>
                ))}

                {roles.length === 0 && (
                    <p className="text-center text-neutral-500 dark:text-neutral-400 py-8">尚無角色</p>
                )}
            </div>
        </div>
    );
}
