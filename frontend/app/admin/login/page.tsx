"use client";

import { useActionState, useCallback, useEffect, useRef, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { KeyRound, Loader2 } from 'lucide-react';
import {
    browserSupportsWebAuthn,
    browserSupportsWebAuthnAutofill,
    startAuthentication,
    startRegistration,
} from '@simplewebauthn/browser';
import { startTokenRefresh } from '@/libs/token-refresh';
import { beginPasskeyRegistration, finishPasskeyRegistration, getPasskeys } from '@/api/auth';

type LoginState = { error: string | null; ok: boolean };

// 升級提示「略過」記憶：30 天內不再煩
const DISMISS_KEY = 'passkey_prompt_dismissed_at';
const DISMISS_MS = 30 * 24 * 60 * 60 * 1000;

function upgradePromptDismissed(): boolean {
    try {
        const at = Number(localStorage.getItem(DISMISS_KEY));
        return Number.isFinite(at) && at > 0 && Date.now() - at < DISMISS_MS;
    } catch {
        return true;
    }
}

async function passkeyLoginOnce(useAutofill: boolean): Promise<void> {
    const beginRes = await fetch('/api/auth/passkey/login/begin', { method: 'POST' });
    if (!beginRes.ok) throw new Error(`begin ${beginRes.status}`);
    const { auth_id, options } = await beginRes.json();

    const credential = await startAuthentication({
        optionsJSON: options.publicKey,
        useBrowserAutofill: useAutofill,
    });

    const finishRes = await fetch('/api/auth/passkey/login/finish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ auth_id, credential }),
    });
    if (!finishRes.ok) {
        const err = new Error(`finish ${finishRes.status}`) as Error & { status: number };
        err.status = finishRes.status;
        throw err;
    }
}

export default function Login() {
    const router = useRouter();
    const searchParams = useSearchParams();
    // open-redirect 防護：只收站內相對路徑（同 OAuth callback 的規則）
    const rawRedirect = searchParams.get('redirect');
    const redirectUrl = rawRedirect?.startsWith('/') && !rawRedirect.startsWith('//') ? rawRedirect : '/admin';

    const [passkeyError, setPasskeyError] = useState<string | null>(null);
    const [passkeyPending, setPasskeyPending] = useState(false);
    const [showUpgrade, setShowUpgrade] = useState(false);
    const [upgradePending, setUpgradePending] = useState(false);
    const [upgradeError, setUpgradeError] = useState<string | null>(null);
    const labelRef = useRef<HTMLInputElement>(null);

    const completeLogin = useCallback(() => {
        startTokenRefresh();
        router.push(redirectUrl);
    }, [router, redirectUrl]);

    // Conditional UI：掛載即備妥挑戰，使用者點帳號欄 autofill 選 passkey 即完成登入。
    // 挑戰 5 分鐘過期（finish 401）→ 重新掛一輪讓下次選取有效
    useEffect(() => {
        let active = true;
        (async () => {
            if (!(await browserSupportsWebAuthnAutofill())) return;
            for (let attempt = 0; active && attempt < 2; attempt++) {
                try {
                    await passkeyLoginOnce(true);
                    if (active) completeLogin();
                    return;
                } catch (e) {
                    const err = e as Error & { status?: number };
                    // StrictMode double-effect / 之後啟動的 modal ceremony 都會 cancel 這輪，屬預期
                    if (err.name === 'AbortError' || err.name === 'NotAllowedError') return;
                    if (err.status !== 401) return; // 非挑戰過期不重試
                }
            }
        })();
        return () => { active = false; };
    }, [completeLogin]);

    const handlePasskeyButton = async () => {
        setPasskeyError(null);
        setPasskeyPending(true);
        try {
            await passkeyLoginOnce(false);
            completeLogin();
        } catch (e) {
            const err = e as Error & { status?: number };
            if (err.name !== 'AbortError' && err.name !== 'NotAllowedError') {
                setPasskeyError(err.status === 401 ? 'Passkey 驗證失敗，請再試一次' : '登入失敗，請稍後再試或改用密碼');
            }
        } finally {
            setPasskeyPending(false);
        }
    };

    const [state, formAction, pending] = useActionState<LoginState, FormData>(
        async (_prev, formData) => {
            const name = formData.get('name');
            const password = formData.get('password');

            try {
                const res = await fetch('/api/auth/login', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ name, password }),
                });

                if (!res.ok) {
                    const data = await res.json();
                    return { error: data.error || '登入失敗', ok: false };
                }

                return { error: null, ok: true };
            } catch {
                return { error: '網路錯誤，請稍後再試', ok: false };
            }
        },
        { error: null, ok: false }
    );

    const error = state.error;

    // 密碼登入成功 → 沒有 passkey 且未略過提示時，先問要不要升級，否則照舊導向
    useEffect(() => {
        if (!state.ok) return;
        let active = true;
        (async () => {
            try {
                if (browserSupportsWebAuthn() && !upgradePromptDismissed()) {
                    const passkeys = await getPasskeys();
                    if (active && passkeys.length === 0) {
                        setShowUpgrade(true);
                        return;
                    }
                }
            } catch { /* 提示失敗不擋登入 */ }
            if (active) completeLogin();
        })();
        return () => { active = false; };
    }, [state.ok, completeLogin]);

    const handleUpgradeCreate = async () => {
        setUpgradeError(null);
        setUpgradePending(true);
        try {
            const options = await beginPasskeyRegistration();
            const credential = await startRegistration({ optionsJSON: options.publicKey });
            const label = labelRef.current?.value.trim() || '我的裝置';
            await finishPasskeyRegistration(credential, label);
            completeLogin();
        } catch (e) {
            const err = e as Error & { status?: number };
            if (err.name === 'NotAllowedError' || err.name === 'AbortError') {
                setUpgradeError(null); // 使用者取消，不當錯誤
            } else if (err.status === 409) {
                setUpgradeError('此裝置已註冊過 passkey');
            } else {
                setUpgradeError('建立失敗，可稍後至「Passkey 管理」再試');
            }
        } finally {
            setUpgradePending(false);
        }
    };

    const handleUpgradeSkip = () => {
        try {
            localStorage.setItem(DISMISS_KEY, String(Date.now()));
        } catch { /* ignore */ }
        completeLogin();
    };

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto flex justify-center items-start">
            <div className="w-full max-w-md p-8 space-y-6 bg-white dark:bg-neutral-800 rounded-lg shadow-md">
                {showUpgrade ? (
                    <div className="space-y-4">
                        <h2 className="text-2xl font-bold text-center text-neutral-800 dark:text-neutral-100">建立 Passkey</h2>
                        <p className="text-sm text-neutral-600 dark:text-neutral-400">
                            使用指紋、臉部辨識或裝置密碼登入，下次免輸入密碼。
                        </p>
                        <div>
                            <label htmlFor="passkey-label" className="block text-sm font-medium text-neutral-700 dark:text-neutral-300">名稱</label>
                            <input
                                type="text"
                                id="passkey-label"
                                ref={labelRef}
                                defaultValue="我的裝置"
                                maxLength={64}
                                className="w-full px-4 py-2 mt-1 text-neutral-900 dark:text-neutral-100 bg-white dark:bg-neutral-700 border dark:border-neutral-600 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500"
                            />
                        </div>
                        {upgradeError && <p className="text-sm text-red-500">{upgradeError}</p>}
                        <button
                            type="button"
                            onClick={handleUpgradeCreate}
                            disabled={upgradePending}
                            className={`w-full px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700 focus:ring-4 focus:ring-primary-500 focus:ring-opacity-50 ${upgradePending ? 'opacity-50 cursor-not-allowed' : ''}`}
                        >
                            {upgradePending ? (
                                <span className="flex items-center justify-center gap-2"><Loader2 className="w-4 h-4 animate-spin" />建立中...</span>
                            ) : (
                                <span className="flex items-center justify-center gap-2"><KeyRound className="w-4 h-4" />建立 Passkey</span>
                            )}
                        </button>
                        <button
                            type="button"
                            onClick={handleUpgradeSkip}
                            disabled={upgradePending}
                            className="w-full px-4 py-2 text-neutral-600 dark:text-neutral-300 hover:text-neutral-800 dark:hover:text-neutral-100 text-sm"
                        >
                            略過（30 天內不再詢問）
                        </button>
                    </div>
                ) : (
                    <>
                        <h2 className="text-2xl font-bold text-center text-neutral-800 dark:text-neutral-100">Login</h2>
                        <form action={formAction} className="space-y-4">
                            <div>
                                <label htmlFor="name" className="block text-sm font-medium text-neutral-700 dark:text-neutral-300">名稱</label>
                                <input type="text" id="name" name="name" autoComplete="username webauthn" className="w-full px-4 py-2 mt-1 text-neutral-900 dark:text-neutral-100 bg-white dark:bg-neutral-700 border dark:border-neutral-600 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500" placeholder="輸入管理員名稱" required />
                            </div>
                            <div>
                                <label htmlFor="password" className="block text-sm font-medium text-neutral-700 dark:text-neutral-300">Password</label>
                                <input type="password" id="password" name="password" autoComplete="current-password" className="w-full px-4 py-2 mt-1 text-neutral-900 dark:text-neutral-100 bg-white dark:bg-neutral-700 border dark:border-neutral-600 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500" placeholder="Enter your password" required />
                            </div>
                            {error && <p className="text-sm text-red-500">{error}</p>}
                            <button
                                type="submit"
                                disabled={pending}
                                className={`w-full px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700 focus:ring-4 focus:ring-primary-500 focus:ring-opacity-50 ${pending ? 'opacity-50 cursor-not-allowed' : ''}`}
                            >
                                {pending ? (
                                    <span className="flex items-center justify-center gap-2"><Loader2 className="w-4 h-4 animate-spin" />Logging in...</span>
                                ) : 'Login'}
                            </button>
                        </form>
                        <div className="flex items-center gap-3">
                            <div className="flex-1 border-t dark:border-neutral-600" />
                            <span className="text-xs text-neutral-400">或</span>
                            <div className="flex-1 border-t dark:border-neutral-600" />
                        </div>
                        {passkeyError && <p className="text-sm text-red-500">{passkeyError}</p>}
                        <button
                            type="button"
                            onClick={handlePasskeyButton}
                            disabled={passkeyPending}
                            className={`w-full px-4 py-2 border border-primary-600 text-primary-600 dark:text-primary-400 dark:border-primary-400 rounded-md hover:bg-primary-50 dark:hover:bg-neutral-700 focus:ring-4 focus:ring-primary-500 focus:ring-opacity-50 transition-colors ${passkeyPending ? 'opacity-50 cursor-not-allowed' : ''}`}
                        >
                            {passkeyPending ? (
                                <span className="flex items-center justify-center gap-2"><Loader2 className="w-4 h-4 animate-spin" />驗證中...</span>
                            ) : (
                                <span className="flex items-center justify-center gap-2"><KeyRound className="w-4 h-4" />使用 Passkey 登入</span>
                            )}
                        </button>
                    </>
                )}
            </div>
        </div>
    );
}
