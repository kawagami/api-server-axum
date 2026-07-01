"use client";

import { useEffect, useRef, useState } from "react";
import { useTranslations } from "next-intl";
import { Loader2, Gamepad2, AlertTriangle } from "lucide-react";

// Bevy wasm-bindgen (--target web) 產物固定路徑（見 docs/bevy-game-spec.md §3.2）
// 放在 public/games/metal-slug/，瀏覽器直接 fetch，不經 bundler。
const GLUE_URL = "/games/metal-slug/game.js";
const CANVAS_ID = "bevy-canvas";

type Status = "loading" | "running" | "error";

export default function MetalSlugGame() {
    const t = useTranslations("Games");
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const startedRef = useRef(false); // StrictMode dev double-mount 防重入
    const [status, setStatus] = useState<Status>("loading");

    useEffect(() => {
        if (startedRef.current) return;
        startedRef.current = true;

        let cancelled = false;

        (async () => {
            try {
                // 絕對路徑靜態資產：繞過 webpack / turbopack 的 import 解析
                const mod = await import(
                    /* webpackIgnore: true */
                    /* turbopackIgnore: true */
                    GLUE_URL
                );
                // wasm-bindgen --target web：default export 是 init()，
                // 載入 game_bg.wasm 並執行 Bevy main（render 進 #bevy-canvas）
                await mod.default();
                if (!cancelled) setStatus("running");
            } catch (e) {
                console.error("[metal-slug] 載入失敗", e);
                if (!cancelled) setStatus("error");
            }
        })();

        return () => {
            cancelled = true;
        };
    }, []);

    return (
        <div className="flex flex-col items-center justify-center h-[calc(100svh-120px)] gap-4">
            <div className="relative w-full max-w-3xl aspect-video rounded-lg overflow-hidden border border-neutral-300 dark:border-neutral-700 bg-neutral-900 shadow-lg">
                {/* Bevy render target — init 前必須已存在於 DOM */}
                <canvas
                    id={CANVAS_ID}
                    ref={canvasRef}
                    className="block w-full h-full focus:outline-none"
                    tabIndex={0}
                />

                {status === "loading" && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-neutral-900/90 animate-pulse">
                        <Loader2 className="w-10 h-10 animate-spin text-primary-400" />
                        <p className="text-sm text-neutral-300">{t("loading")}</p>
                    </div>
                )}

                {status === "error" && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-neutral-900/95 px-6 text-center">
                        <AlertTriangle className="w-10 h-10 text-red-400" />
                        <p className="text-sm text-neutral-200">{t("error")}</p>
                    </div>
                )}
            </div>

            <p className="flex items-center gap-2 text-sm text-neutral-500 dark:text-neutral-400">
                <Gamepad2 className="w-4 h-4" />
                {t("hint")}
            </p>
        </div>
    );
}
