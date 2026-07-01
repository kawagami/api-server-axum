"use client";

import { clearSession } from "@/app/admin/login/actions";
import { LogOut } from "lucide-react";

export default function LogoutButton() {
    async function handleLogout() {
        localStorage.removeItem('token');
        await clearSession();
    }

    return (
        <button
            onClick={handleLogout}
            className="flex items-center gap-1 px-3 py-1 text-sm text-red-600 border border-red-300 rounded hover:bg-red-50 dark:text-red-400 dark:border-red-700 dark:hover:bg-red-950"
        >
            <LogOut size={14} />
            登出
        </button>
    );
}
