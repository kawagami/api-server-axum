"use client";

import { useState, useTransition } from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';
import { setUserRoles } from '@/app/admin/(main)/roles/actions';
import type { Role } from '@/types';

interface Props {
    userId: string;
    userName: string;
    initialRoles: Role[];
    allRoles: Role[];
}

export default function UserRolesPanel({ userId, userName, initialRoles, allRoles }: Props) {
    const [expanded, setExpanded] = useState(false);
    const [assignedIds, setAssignedIds] = useState<number[]>(initialRoles.map(r => r.id));
    const [isPending, startTransition] = useTransition();

    function toggle(roleId: number) {
        const next = assignedIds.includes(roleId)
            ? assignedIds.filter(id => id !== roleId)
            : [...assignedIds, roleId];

        setAssignedIds(next);
        startTransition(async () => {
            await setUserRoles(userId, next);
        });
    }

    return (
        <div>
            <button
                onClick={() => setExpanded(prev => !prev)}
                className="flex items-center gap-1 text-sm text-primary-600 dark:text-primary-400 hover:underline"
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
                                className={`px-2 py-0.5 text-xs rounded-full border transition-colors ${
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
        </div>
    );
}
