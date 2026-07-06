"use client";

import { useState, useTransition } from "react";
import { UserPlus } from "lucide-react";
import { createUser } from "./actions";
import type { Role } from "@/types";

interface Props {
    allRoles: Role[];
    defaultRoleIds: number[];
}

export default function CreateUserForm({ allRoles, defaultRoleIds }: Props) {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("");
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [roleIds, setRoleIds] = useState<number[]>(defaultRoleIds);
    const [error, setError] = useState("");
    const [isPending, startTransition] = useTransition();

    function reset() {
        setName("");
        setEmail("");
        setPassword("");
        setRoleIds(defaultRoleIds);
        setError("");
    }

    function toggleRole(id: number) {
        setRoleIds(prev => (prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id]));
    }

    function submit() {
        setError("");
        startTransition(async () => {
            try {
                await createUser({ name, email, password, role_ids: roleIds });
                reset();
                setOpen(false);
            } catch (err) {
                setError((err as Error).message);
            }
        });
    }

    if (!open) {
        return (
            <button
                onClick={() => setOpen(true)}
                className="mb-4 inline-flex items-center gap-1.5 px-3 py-2 text-sm font-medium bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
            >
                <UserPlus size={16} />
                新增管理員
            </button>
        );
    }

    return (
        <div className="mb-4 bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4 space-y-3">
            <h2 className="text-sm font-semibold text-neutral-800 dark:text-neutral-200">新增管理員</h2>

            <div className="grid gap-3 sm:grid-cols-3">
                <input
                    type="text"
                    value={name}
                    onChange={e => setName(e.target.value)}
                    placeholder="名稱（登入帳號）"
                    className="px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-600 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
                <input
                    type="email"
                    value={email}
                    onChange={e => setEmail(e.target.value)}
                    placeholder="Email（選填）"
                    className="px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-600 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
                <input
                    type="password"
                    value={password}
                    onChange={e => setPassword(e.target.value)}
                    placeholder="密碼"
                    autoComplete="new-password"
                    className="px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-600 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
            </div>

            <div>
                <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-1.5">角色</p>
                <div className="flex flex-wrap gap-1">
                    {allRoles.map(role => {
                        const active = roleIds.includes(role.id);
                        return (
                            <button
                                key={role.id}
                                type="button"
                                onClick={() => toggleRole(role.id)}
                                className={`px-2 py-0.5 text-xs rounded-full border transition-colors ${
                                    active
                                        ? "bg-primary-100 border-primary-400 text-primary-700 dark:bg-primary-900 dark:border-primary-500 dark:text-primary-300"
                                        : "bg-neutral-100 border-neutral-300 text-neutral-600 dark:bg-neutral-800 dark:border-neutral-600 dark:text-neutral-400"
                                }`}
                            >
                                {role.name}
                            </button>
                        );
                    })}
                </div>
            </div>

            {error && <p className="text-xs text-red-500">{error}</p>}

            <div className="flex gap-2">
                <button
                    onClick={submit}
                    disabled={isPending || !name || !password}
                    className="px-4 py-2 text-sm font-medium bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white rounded-lg transition-colors"
                >
                    {isPending ? "建立中..." : "建立"}
                </button>
                <button
                    onClick={() => {
                        reset();
                        setOpen(false);
                    }}
                    disabled={isPending}
                    className="px-4 py-2 text-sm font-medium text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 rounded-lg transition-colors"
                >
                    取消
                </button>
            </div>
        </div>
    );
}
