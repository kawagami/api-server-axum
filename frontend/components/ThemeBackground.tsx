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

const containerStyle: React.CSSProperties = {
    position: "fixed",
    inset: 0,
    zIndex: 0,
    overflow: "hidden",
    pointerEvents: "none",
};

/** forest：落葉飄下 */
function Leaves() {
    return (
        <>
            <style>{`
                @keyframes leaf-fall {
                    0%   { transform: translateY(-8vh); opacity: 0; }
                    8%   { opacity: 1; }
                    85%  { opacity: 0.7; }
                    100% { transform: translateY(105vh); opacity: 0; }
                }
                @keyframes leaf-sway {
                    0%   { transform: translateX(-14px) rotate(-24deg); }
                    50%  { transform: translateX(14px) rotate(20deg); }
                    100% { transform: translateX(-14px) rotate(-24deg); }
                }
                .leaf-wrap {
                    position: absolute;
                    top: 0;
                    will-change: transform, opacity;
                    animation: leaf-fall linear infinite;
                    animation-fill-mode: both;
                    pointer-events: none;
                }
                .leaf {
                    display: block;
                    animation: leaf-sway ease-in-out infinite;
                }
                .leaf .blade { fill: rgb(var(--primary-500) / 0.45); }
                .leaf .vein  { stroke: rgb(var(--primary-700) / 0.5); }
                .dark .leaf .blade { fill: rgb(var(--primary-400) / 0.28); }
                .dark .leaf .vein  { stroke: rgb(var(--primary-100) / 0.32); }
                @media (prefers-reduced-motion: reduce) {
                    .leaf-wrap { animation: none; opacity: 0; }
                }
            `}</style>
            {PARTICLES.map((p, i) => (
                <div
                    key={i}
                    className="leaf-wrap"
                    style={{
                        left: `${p.left}%`,
                        animationDelay: `${p.delay}s`,
                        animationDuration: `${p.duration}s`,
                    }}
                >
                    <svg
                        className="leaf"
                        width={p.size}
                        height={p.size}
                        viewBox="0 0 32 32"
                        style={{
                            animationDuration: `${p.swayDuration}s`,
                            // 負延遲讓每片葉子從搖擺週期的不同相位開始
                            animationDelay: `-${(i % 5) * 0.7}s`,
                        }}
                    >
                        <path className="blade" d="M3 23 C 5 9, 18 2, 29 5 C 27 18, 13 27, 4 25 Z" />
                        <path className="vein" d="M6 22 C 13 16, 21 10, 27 7" fill="none" strokeWidth="2" strokeLinecap="round" />
                    </svg>
                </div>
            ))}
        </>
    );
}

/** ocean：氣泡上浮 */
function Bubbles() {
    return (
        <>
            <style>{`
                @keyframes bubble-rise {
                    0%   { transform: translateY(105vh); opacity: 0; }
                    8%   { opacity: 1; }
                    85%  { opacity: 0.7; }
                    100% { transform: translateY(-8vh); opacity: 0; }
                }
                @keyframes bubble-sway {
                    0%   { transform: translateX(-10px); }
                    50%  { transform: translateX(10px); }
                    100% { transform: translateX(-10px); }
                }
                .bubble-wrap {
                    position: absolute;
                    top: 0;
                    will-change: transform, opacity;
                    animation: bubble-rise linear infinite;
                    animation-fill-mode: both;
                    pointer-events: none;
                }
                .bubble {
                    display: block;
                    border-radius: 50%;
                    animation: bubble-sway ease-in-out infinite;
                    border: 2px solid rgb(var(--primary-500) / 0.4);
                    background: radial-gradient(circle at 30% 30%, rgb(255 255 255 / 0.35), rgb(var(--primary-300) / 0.12));
                }
                .dark .bubble {
                    border-color: rgb(var(--primary-400) / 0.3);
                    background: radial-gradient(circle at 30% 30%, rgb(var(--primary-100) / 0.12), rgb(var(--primary-600) / 0.08));
                    box-shadow: 0 0 12px rgb(var(--primary-400) / 0.15);
                }
                @media (prefers-reduced-motion: reduce) {
                    .bubble-wrap { animation: none; opacity: 0; }
                }
            `}</style>
            {PARTICLES.map((p, i) => (
                <div
                    key={i}
                    className="bubble-wrap"
                    style={{
                        left: `${p.left}%`,
                        animationDelay: `${p.delay}s`,
                        animationDuration: `${p.duration}s`,
                    }}
                >
                    <div
                        className="bubble"
                        style={{
                            width: p.size,
                            height: p.size,
                            animationDuration: `${p.swayDuration}s`,
                            animationDelay: `-${(i % 5) * 0.7}s`,
                        }}
                    />
                </div>
            ))}
        </>
    );
}

/** sky：雲朵橫飄 */
function Clouds() {
    return (
        <>
            <style>{`
                @keyframes cloud-drift {
                    0%   { transform: translateX(-15vw); opacity: 0; }
                    8%   { opacity: 1; }
                    85%  { opacity: 0.8; }
                    100% { transform: translateX(112vw); opacity: 0; }
                }
                @keyframes cloud-bob {
                    0%   { transform: translateY(-6px); }
                    50%  { transform: translateY(6px); }
                    100% { transform: translateY(-6px); }
                }
                .cloud-wrap {
                    position: absolute;
                    left: 0;
                    will-change: transform, opacity;
                    animation: cloud-drift linear infinite;
                    animation-fill-mode: both;
                    pointer-events: none;
                }
                .cloud {
                    display: block;
                    animation: cloud-bob ease-in-out infinite;
                }
                .cloud .puff { fill: rgb(var(--primary-300) / 0.35); }
                .dark .cloud .puff { fill: rgb(var(--primary-200) / 0.1); }
                @media (prefers-reduced-motion: reduce) {
                    .cloud-wrap { animation: none; opacity: 0; }
                }
            `}</style>
            {PARTICLES.map((p, i) => (
                <div
                    key={i}
                    className="cloud-wrap"
                    style={{
                        // 雲分佈在畫面上 75% 高度區間，橫飄速度放慢（落下參數 ×3）
                        top: `${(p.left * 0.75)}%`,
                        animationDelay: `${p.delay * 2}s`,
                        animationDuration: `${p.duration * 3}s`,
                    }}
                >
                    <svg
                        className="cloud"
                        width={p.size * 2.6}
                        height={p.size * 1.6}
                        viewBox="0 0 52 32"
                        style={{
                            animationDuration: `${p.swayDuration * 2}s`,
                            animationDelay: `-${(i % 5) * 1.1}s`,
                        }}
                    >
                        <path
                            className="puff"
                            d="M14 26 a8 8 0 0 1 -1 -16 a10 10 0 0 1 19 -3 a8 8 0 0 1 11 7 a6.5 6.5 0 0 1 -2 12 Z"
                        />
                    </svg>
                </div>
            ))}
        </>
    );
}

/** sakura：花瓣飄下（落葉變體，圓潤花瓣形） */
function Petals() {
    return (
        <>
            <style>{`
                @keyframes petal-fall {
                    0%   { transform: translateY(-8vh); opacity: 0; }
                    8%   { opacity: 1; }
                    85%  { opacity: 0.75; }
                    100% { transform: translateY(105vh); opacity: 0; }
                }
                @keyframes petal-sway {
                    0%   { transform: translateX(-16px) rotate(-30deg); }
                    50%  { transform: translateX(16px) rotate(25deg); }
                    100% { transform: translateX(-16px) rotate(-30deg); }
                }
                .petal-wrap {
                    position: absolute;
                    top: 0;
                    will-change: transform, opacity;
                    animation: petal-fall linear infinite;
                    animation-fill-mode: both;
                    pointer-events: none;
                }
                .petal {
                    display: block;
                    animation: petal-sway ease-in-out infinite;
                }
                .petal .blade { fill: rgb(var(--primary-400) / 0.5); }
                .dark .petal .blade { fill: rgb(var(--primary-300) / 0.3); }
                @media (prefers-reduced-motion: reduce) {
                    .petal-wrap { animation: none; opacity: 0; }
                }
            `}</style>
            {PARTICLES.map((p, i) => (
                <div
                    key={i}
                    className="petal-wrap"
                    style={{
                        left: `${p.left}%`,
                        animationDelay: `${p.delay}s`,
                        animationDuration: `${p.duration}s`,
                    }}
                >
                    <svg
                        className="petal"
                        width={p.size}
                        height={p.size}
                        viewBox="0 0 32 32"
                        style={{
                            animationDuration: `${p.swayDuration}s`,
                            animationDelay: `-${(i % 5) * 0.7}s`,
                        }}
                    >
                        {/* 花瓣：上窄下圓、頂端有小凹口 */}
                        <path className="blade" d="M16 3 C 9 11, 9 22, 16 29 C 23 22, 23 11, 16 3 Z M16 3 C 17 6, 15 6, 16 3 Z" />
                    </svg>
                </div>
            ))}
        </>
    );
}

/** sunset：餘燼緩緩上浮（氣泡變體，暖色實心小點 + 微光） */
function Embers() {
    return (
        <>
            <style>{`
                @keyframes ember-rise {
                    0%   { transform: translateY(105vh); opacity: 0; }
                    10%  { opacity: 1; }
                    85%  { opacity: 0.6; }
                    100% { transform: translateY(-8vh); opacity: 0; }
                }
                @keyframes ember-sway {
                    0%   { transform: translateX(-8px); }
                    50%  { transform: translateX(8px); }
                    100% { transform: translateX(-8px); }
                }
                .ember-wrap {
                    position: absolute;
                    top: 0;
                    will-change: transform, opacity;
                    animation: ember-rise linear infinite;
                    animation-fill-mode: both;
                    pointer-events: none;
                }
                .ember {
                    display: block;
                    border-radius: 50%;
                    animation: ember-sway ease-in-out infinite;
                    background: radial-gradient(circle, rgb(var(--primary-400) / 0.85), rgb(var(--primary-600) / 0.2));
                    box-shadow: 0 0 8px rgb(var(--primary-500) / 0.5);
                }
                .dark .ember {
                    background: radial-gradient(circle, rgb(var(--primary-300) / 0.7), rgb(var(--primary-500) / 0.15));
                    box-shadow: 0 0 10px rgb(var(--primary-400) / 0.4);
                }
                @media (prefers-reduced-motion: reduce) {
                    .ember-wrap { animation: none; opacity: 0; }
                }
            `}</style>
            {PARTICLES.map((p, i) => (
                <div
                    key={i}
                    className="ember-wrap"
                    style={{
                        left: `${p.left}%`,
                        animationDelay: `${p.delay}s`,
                        animationDuration: `${p.duration}s`,
                    }}
                >
                    <div
                        className="ember"
                        style={{
                            // 餘燼比氣泡小一截
                            width: p.size * 0.5,
                            height: p.size * 0.5,
                            animationDuration: `${p.swayDuration}s`,
                            animationDelay: `-${(i % 5) * 0.7}s`,
                        }}
                    />
                </div>
            ))}
        </>
    );
}

const PARTICLE_VARIANTS: Record<SiteTheme, () => React.JSX.Element> = {
    forest: Leaves,
    ocean: Bubbles,
    sky: Clouds,
    sunset: Embers,
    sakura: Petals,
    grape: Bubbles,   // 紫色氣泡上浮（色走 var）
    mono: Clouds,     // 灰雲橫飄（色走 var）
};

export default function ThemeBackground({ theme }: { theme: SiteTheme }) {
    const Particles = PARTICLE_VARIANTS[theme] ?? Leaves;
    return (
        <div aria-hidden="true" style={containerStyle}>
            <Particles />
        </div>
    );
}
