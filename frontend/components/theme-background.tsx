"use client";

import type { SiteTheme } from "@/libs/site-theme";

// size: px、left: %、delay/duration: 主動畫（s）、swayDuration: 搖擺週期（s）
const PARTICLES = [
    { size: 22, left: 5,  delay: 0,   duration: 11, swayDuration: 3.2 },
    { size: 16, left: 12, delay: 2.5, duration: 14, swayDuration: 2.6 },
    { size: 28, left: 20, delay: 5,   duration: 10, swayDuration: 3.8 },
    { size: 13, left: 30, delay: 1,   duration: 16, swayDuration: 2.4 },
    { size: 24, left: 38, delay: 3.5, duration: 12, swayDuration: 3.5 },
    { size: 18, left: 47, delay: 7,   duration: 15, swayDuration: 2.9 },
    { size: 30, left: 55, delay: 1.8, duration: 11, swayDuration: 4.1 },
    { size: 14, left: 63, delay: 6,   duration: 14, swayDuration: 2.5 },
    { size: 26, left: 71, delay: 1.2, duration: 10, swayDuration: 3.6 },
    { size: 19, left: 79, delay: 4.2, duration: 17, swayDuration: 2.8 },
    { size: 27, left: 86, delay: 2,   duration: 12, swayDuration: 3.3 },
    { size: 16, left: 92, delay: 7.5, duration: 15, swayDuration: 2.7 },
    { size: 21, left: 97, delay: 3.8, duration: 11, swayDuration: 3.0 },
    { size: 12, left: 44, delay: 8.5, duration: 18, swayDuration: 2.3 },
    { size: 24, left: 68, delay: 6.5, duration: 13, swayDuration: 3.9 },
];

type Particle = (typeof PARTICLES)[number];

type ParticleConfig = {
    prefix: string;                                              // class：wrap=`${prefix}-wrap`、shape=`${prefix}`；keyframes=`${prefix}-main/-sway`
    main: { from: string; to: string; fadeIn: number; mid: number }; // 主移動：transform 端點、淡入時點（%）、85% 透明度
    sway: [string, string];                                      // 搖擺的 0%/100% 與 50% transform
    shapeCss: string;                                            // 形狀色彩/裝飾 CSS（含 .dark 變體，色一律走 CSS var）
    drift?: boolean;                                             // true=橫飄（clouds）：分佈於 top、放慢（delay ×2/duration ×3/sway ×2/相位 1.1s）
    renderShape: (p: Particle, style: React.CSSProperties) => React.ReactNode; // style 已含 sway 的 duration/負相位 delay
};

/** 共用粒子場：keyframes / 樣板 CSS / PARTICLES map 只寫一次，差異全由 config 提供 */
function ParticleField({ cfg }: { cfg: ParticleConfig }) {
    const { prefix, main, sway, drift } = cfg;
    return (
        <>
            <style>{`
                @keyframes ${prefix}-main {
                    0%   { transform: ${main.from}; opacity: 0; }
                    ${main.fadeIn}%   { opacity: 1; }
                    85%  { opacity: ${main.mid}; }
                    100% { transform: ${main.to}; opacity: 0; }
                }
                @keyframes ${prefix}-sway {
                    0%   { transform: ${sway[0]}; }
                    50%  { transform: ${sway[1]}; }
                    100% { transform: ${sway[0]}; }
                }
                .${prefix}-wrap {
                    position: absolute;
                    ${drift ? "left" : "top"}: 0;
                    will-change: transform, opacity;
                    animation: ${prefix}-main linear infinite;
                    animation-fill-mode: both;
                    pointer-events: none;
                }
                .${prefix} {
                    display: block;
                    animation: ${prefix}-sway ease-in-out infinite;
                }
                ${cfg.shapeCss}
                @media (prefers-reduced-motion: reduce) {
                    .${prefix}-wrap { animation: none; opacity: 0; }
                }
            `}</style>
            {PARTICLES.map((p, i) => (
                <div
                    key={i}
                    className={`${prefix}-wrap`}
                    style={
                        drift
                            ? { top: `${p.left * 0.75}%`, animationDelay: `${p.delay * 2}s`, animationDuration: `${p.duration * 3}s` }
                            : { left: `${p.left}%`, animationDelay: `${p.delay}s`, animationDuration: `${p.duration}s` }
                    }
                >
                    {cfg.renderShape(p, {
                        animationDuration: `${p.swayDuration * (drift ? 2 : 1)}s`,
                        // 負延遲讓每顆粒子從搖擺週期的不同相位開始
                        animationDelay: `-${(i % 5) * (drift ? 1.1 : 0.7)}s`,
                    })}
                </div>
            ))}
        </>
    );
}

/** forest：落葉飄下 */
const leaf: ParticleConfig = {
    prefix: "leaf",
    main: { from: "translateY(-8vh)", to: "translateY(105vh)", fadeIn: 8, mid: 0.7 },
    sway: ["translateX(-14px) rotate(-24deg)", "translateX(14px) rotate(20deg)"],
    shapeCss: `
        .leaf .blade { fill: rgb(var(--primary-500) / 0.45); }
        .leaf .vein  { stroke: rgb(var(--primary-700) / 0.5); }
        .dark .leaf .blade { fill: rgb(var(--primary-400) / 0.28); }
        .dark .leaf .vein  { stroke: rgb(var(--primary-100) / 0.32); }`,
    renderShape: (p, style) => (
        <svg className="leaf" width={p.size} height={p.size} viewBox="0 0 32 32" style={style}>
            <path className="blade" d="M3 23 C 5 9, 18 2, 29 5 C 27 18, 13 27, 4 25 Z" />
            <path className="vein" d="M6 22 C 13 16, 21 10, 27 7" fill="none" strokeWidth="2" strokeLinecap="round" />
        </svg>
    ),
};

/** ocean / grape：氣泡上浮（grape 為紫色氣泡，色走 var） */
const bubble: ParticleConfig = {
    prefix: "bubble",
    main: { from: "translateY(105vh)", to: "translateY(-8vh)", fadeIn: 8, mid: 0.7 },
    sway: ["translateX(-10px)", "translateX(10px)"],
    shapeCss: `
        .bubble {
            border-radius: 50%;
            border: 2px solid rgb(var(--primary-500) / 0.4);
            background: radial-gradient(circle at 30% 30%, rgb(255 255 255 / 0.35), rgb(var(--primary-300) / 0.12));
        }
        .dark .bubble {
            border-color: rgb(var(--primary-400) / 0.3);
            background: radial-gradient(circle at 30% 30%, rgb(var(--primary-100) / 0.12), rgb(var(--primary-600) / 0.08));
            box-shadow: 0 0 12px rgb(var(--primary-400) / 0.15);
        }`,
    renderShape: (p, style) => <div className="bubble" style={{ width: p.size, height: p.size, ...style }} />,
};

/** sky / mono：雲朵橫飄（分佈在畫面上 75% 高度區間、速度放慢；mono 為灰雲，色走 var） */
const cloud: ParticleConfig = {
    prefix: "cloud",
    drift: true,
    main: { from: "translateX(-15vw)", to: "translateX(112vw)", fadeIn: 8, mid: 0.8 },
    sway: ["translateY(-6px)", "translateY(6px)"],
    shapeCss: `
        .cloud .puff { fill: rgb(var(--primary-300) / 0.35); }
        .dark .cloud .puff { fill: rgb(var(--primary-200) / 0.1); }`,
    renderShape: (p, style) => (
        <svg className="cloud" width={p.size * 2.6} height={p.size * 1.6} viewBox="0 0 52 32" style={style}>
            <path className="puff" d="M14 26 a8 8 0 0 1 -1 -16 a10 10 0 0 1 19 -3 a8 8 0 0 1 11 7 a6.5 6.5 0 0 1 -2 12 Z" />
        </svg>
    ),
};

/** sakura：花瓣飄下（落葉變體，圓潤花瓣形：上窄下圓、頂端有小凹口） */
const petal: ParticleConfig = {
    prefix: "petal",
    main: { from: "translateY(-8vh)", to: "translateY(105vh)", fadeIn: 8, mid: 0.75 },
    sway: ["translateX(-16px) rotate(-30deg)", "translateX(16px) rotate(25deg)"],
    shapeCss: `
        .petal .blade { fill: rgb(var(--primary-400) / 0.5); }
        .dark .petal .blade { fill: rgb(var(--primary-300) / 0.3); }`,
    renderShape: (p, style) => (
        <svg className="petal" width={p.size} height={p.size} viewBox="0 0 32 32" style={style}>
            <path className="blade" d="M16 3 C 9 11, 9 22, 16 29 C 23 22, 23 11, 16 3 Z M16 3 C 17 6, 15 6, 16 3 Z" />
        </svg>
    ),
};

/** sunset：餘燼緩緩上浮（氣泡變體，暖色實心小點 + 微光，比氣泡小一截） */
const ember: ParticleConfig = {
    prefix: "ember",
    main: { from: "translateY(105vh)", to: "translateY(-8vh)", fadeIn: 10, mid: 0.6 },
    sway: ["translateX(-8px)", "translateX(8px)"],
    shapeCss: `
        .ember {
            border-radius: 50%;
            background: radial-gradient(circle, rgb(var(--primary-400) / 0.85), rgb(var(--primary-600) / 0.2));
            box-shadow: 0 0 8px rgb(var(--primary-500) / 0.5);
        }
        .dark .ember {
            background: radial-gradient(circle, rgb(var(--primary-300) / 0.7), rgb(var(--primary-500) / 0.15));
            box-shadow: 0 0 10px rgb(var(--primary-400) / 0.4);
        }`,
    renderShape: (p, style) => <div className="ember" style={{ width: p.size * 0.5, height: p.size * 0.5, ...style }} />,
};

const PARTICLE_VARIANTS: Record<SiteTheme, ParticleConfig> = {
    forest: leaf,
    ocean: bubble,
    sky: cloud,
    sunset: ember,
    sakura: petal,
    grape: bubble,   // 紫色氣泡上浮（色走 var）
    mono: cloud,     // 灰雲橫飄（色走 var）
};

const containerStyle: React.CSSProperties = {
    position: "fixed",
    inset: 0,
    zIndex: 0,
    overflow: "hidden",
    pointerEvents: "none",
};

export default function ThemeBackground({ theme }: { theme: SiteTheme }) {
    return (
        <div aria-hidden="true" style={containerStyle}>
            <ParticleField cfg={PARTICLE_VARIANTS[theme] ?? leaf} />
        </div>
    );
}
