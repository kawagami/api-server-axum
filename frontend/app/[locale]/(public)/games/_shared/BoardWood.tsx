// 木紋棋盤與棋子陰影的共用 SVG 素材（圍棋、象棋）。
// 只放 defs 與底層 rect；格線、棋子各盤面自己畫。id 由呼叫端的 useId 產生，避免同頁多盤撞 id。

/// 格線、星位、河界字共用的墨色
export const BOARD_INK = '#4a3418';

export function WoodDefs({ id }: { id: (name: string) => string }) {
    return (
        <>
            <linearGradient id={id('wood')} x1="0" y1="0" x2="1" y2="1">
                <stop offset="0" stopColor="#ecc983" />
                <stop offset="0.55" stopColor="#dfb064" />
                <stop offset="1" stopColor="#cf9a4c" />
            </linearGradient>
            {/* 木紋：橫向拉長的 fractal noise，只取 R 通道當 alpha、上色成深褐 */}
            <filter id={id('grain')} x="0" y="0" width="100%" height="100%">
                <feTurbulence type="fractalNoise" baseFrequency="0.005 0.14" numOctaves="3" seed="11" />
                <feColorMatrix type="matrix"
                    values="0 0 0 0 0.42  0 0 0 0 0.26  0 0 0 0 0.09  2.2 0 0 0 -0.95" />
            </filter>
            <radialGradient id={id('vignette')} cx="0.5" cy="0.45" r="0.75">
                <stop offset="0.6" stopColor="#3b2410" stopOpacity="0" />
                <stop offset="1" stopColor="#3b2410" stopOpacity="0.28" />
            </radialGradient>
            {/* 棋子陰影：漸層圓取代 feDropShadow，上百顆子不會每顆跑一次濾鏡 */}
            <radialGradient id={id('shadow')}>
                <stop offset="0.55" stopColor="#1c1006" stopOpacity="0.5" />
                <stop offset="1" stopColor="#1c1006" stopOpacity="0" />
            </radialGradient>
        </>
    );
}

/// 底色 → 木紋 → 邊緣暗角 → 深色模式壓暗一層
export function WoodSurface({ id, w, h }: { id: (name: string) => string; w: number; h: number }) {
    return (
        <>
            <rect width={w} height={h} fill={`url(#${id('wood')})`} />
            <rect width={w} height={h} filter={`url(#${id('grain')})`} opacity={0.45} />
            <rect width={w} height={h} fill={`url(#${id('vignette')})`} />
            <rect width={w} height={h} className="fill-black opacity-0 dark:opacity-30" />
        </>
    );
}

/// 棋子陰影：光源左上，往右下偏
export function PieceShadow({ id, x, y, r, lift = 0 }: {
    id: (name: string) => string; x: number; y: number; r: number; lift?: number;
}) {
    return <circle cx={x + 1.6 + lift} cy={y + 2.2 + lift * 1.4} r={r + 1.2 + lift} fill={`url(#${id('shadow')})`} />;
}
