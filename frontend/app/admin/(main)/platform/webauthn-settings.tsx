"use client";

import { useState } from "react";
import { updateSetting } from "../settings/actions";

interface Props {
    initialRpId: string;
    initialRpOrigin: string;
}

// Passkey（WebAuthn）RP 設定：存 app_settings 平台保留 key，PATCH 即時生效（後端熱重載）。
// rp_id 一旦有使用者建立 passkey 就不可再改（改 = 既有 passkey 全數作廢），故放平台頁不放一般設定。
export default function WebauthnSettings({ initialRpId, initialRpOrigin }: Props) {
    const [rpId, setRpId] = useState(initialRpId);
    const [rpOrigin, setRpOrigin] = useState(initialRpOrigin);
    const [saved, setSaved] = useState({ rpId: initialRpId, rpOrigin: initialRpOrigin });
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState(false);

    const dirty = rpId !== saved.rpId || rpOrigin !== saved.rpOrigin;

    // 配對規則在這裡驗（兩值同表單）：rp_id 必須是 origin hostname 的有效網域。
    // 後端 PATCH 刻意只驗單值形狀——用另一半現值驗配對會讓「整組換新網域」死鎖存不進去
    function pairError(id: string, origin: string): string | null {
        let hostname: string;
        try {
            const url = new URL(origin);
            if (url.protocol !== "http:" && url.protocol !== "https:") throw new Error();
            hostname = url.hostname;
        } catch {
            return "Origin 必須是合法 http(s) URL（如 https://kawa.homes）";
        }
        if (!id || id.includes("/") || id.includes(":") || id.includes(" ")) {
            return "RP ID 必須是裸網域（如 kawa.homes，不含 scheme / port / 路徑）";
        }
        if (hostname !== id && !hostname.endsWith(`.${id}`)) {
            return `RP ID（${id}）必須是 Origin 網域（${hostname}）本身或其上層網域`;
        }
        return null;
    }

    async function save() {
        if (saving || !dirty) return;
        setError(null);
        setSuccess(false);

        const id = rpId.trim();
        const origin = rpOrigin.trim();
        const invalid = pairError(id, origin);
        if (invalid) {
            setError(invalid);
            return;
        }

        setSaving(true);
        try {
            if (origin !== saved.rpOrigin) {
                await updateSetting("webauthn_rp_origin", origin);
            }
            if (id !== saved.rpId) {
                await updateSetting("webauthn_rp_id", id);
            }
            setSaved({ rpId: id, rpOrigin: origin });
            setSuccess(true);
        } catch (err) {
            setError((err as Error).message);
        } finally {
            setSaving(false);
        }
    }

    const inputClass = "w-full px-3 py-2 rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-primary-500";

    return (
        <div className="bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4 mb-6">
            <p className="text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-3">
                Passkey（WebAuthn）
                <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono">webauthn_rp_id / webauthn_rp_origin</span>
            </p>

            <div className="space-y-3">
                <div>
                    <label className="block text-xs text-neutral-500 dark:text-neutral-400 mb-1">
                        RP ID（綁定網域，如 kawa.homes）
                    </label>
                    <input
                        type="text"
                        value={rpId}
                        disabled={saving}
                        onChange={(e) => setRpId(e.target.value)}
                        className={inputClass}
                    />
                </div>
                <div>
                    <label className="block text-xs text-neutral-500 dark:text-neutral-400 mb-1">
                        Origin（登入頁所在的前端來源，如 https://kawa.homes）
                    </label>
                    <input
                        type="text"
                        value={rpOrigin}
                        disabled={saving}
                        onChange={(e) => setRpOrigin(e.target.value)}
                        className={inputClass}
                    />
                </div>
            </div>

            <div className="flex items-center gap-3 mt-4">
                <button
                    onClick={save}
                    disabled={saving || !dirty}
                    className="px-4 py-2 text-sm font-medium bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white rounded-lg transition-colors"
                >
                    {saving ? "儲存中..." : "儲存"}
                </button>
                {dirty && !saving && (
                    <span className="text-xs text-neutral-500 dark:text-neutral-400">有未儲存的變更</span>
                )}
                {success && !dirty && (
                    <span className="text-xs text-green-600 dark:text-green-400">已儲存，即時生效</span>
                )}
            </div>

            {error && <p className="mt-2 text-xs text-red-500">{error}</p>}
            <p className="mt-2 text-xs text-amber-600 dark:text-amber-400">
                ⚠️ 已有使用者建立 passkey 後，RP ID 不可再改——改了所有既有 passkey 立即作廢（密碼登入不受影響）。
            </p>
            <p className="mt-1 text-xs text-neutral-400 dark:text-neutral-500">
                RP ID 必須是 Origin 的有效網域（設註冊網域可涵蓋所有子網域）；設定不完整時 passkey 登入/註冊不可用。
            </p>
        </div>
    );
}
