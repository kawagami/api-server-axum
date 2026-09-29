"use client";

import { useState, useTransition } from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';
import { setUserRoles } from '@/app/admin/(main)/roles/actions';
import ErrorBanner from '@/components/admin/error-banner';
import type { Role } from '@/types';

interface Props {
    userId: number;
    userName: string;
    initialRoles: Role[];
    allRoles: Role[];
}

export default function UserRolesPanel({ userId, userName, initialRoles, allRoles }: Props) {
    const [expanded, setExpanded] = useState(false);
    const [assignedIds, setAssignedIds] = useState<number[]>(initialRoles.map(r => r.id));
    const [isPending, startTransition] = useTransition();
    const [error, setError] = useState<string | null>(null);

    function toggle(roleId: number) {
        const prev = assignedIds;
        const next = prev.includes(roleId)
            ? prev.filter(id => id !== roleId)
            : [...prev, roleId];

        // 先亮起來再送；被後端擋下（改自己的角色、指派超出自己權限的角色…）就退回原狀，
        // 不然畫面顯示已指派、實際上什麼都沒改
        setAssignedIds(next);
        setError(null);
        startTransition(async () => {
            const res = await setUserRoles(userId, next);
            if (!res.ok) {
                setAssignedIds(prev);
                setError(res.message ?? '角色變更失敗，請稍後再試');
            }
        });
    }

    return (
        <div>
            <button
                onClick={() => setExpanded(prev => !prev)}
                className="flex items-center gap-1 py-1.5 text-sm text-primary-600 dark:text-primary-400 hover:underline"
            >
                {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                {assignedIds.length > 0 ? `${assignedIds.length} role(s)` : '無'}
            </button>

            {expanded && (
                <div className="flex flex-wrap gap-1 mt-2">
                    {allRoles.map(role => {
                        const active = assignedIds.includes(role.id);
                        return (
                            <button
                                key={role.id}
                                onClick={() => toggle(role.id)}
                                disabled={isPending}
                                className={`px-2.5 py-1.5 text-xs rounded-full border transition-colors ${
                                    active
                                        ? 'bg-primary-100 border-primary-400 text-primary-700 dark:bg-primary-900 dark:border-primary-500 dark:text-primary-300'
                                        : 'bg-neutral-100 border-neutral-300 text-neutral-600 dark:bg-neutral-800 dark:border-neutral-600 dark:text-neutral-400'
                                }`}
                            >
                                {role.name}
                            </button>
                        );
                    })}
                </div>
            )}
            {error && <div className="mt-2"><ErrorBanner message={error} /></div>}
        </div>
    );
}
