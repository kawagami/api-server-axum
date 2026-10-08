// 象棋子造型（象棋、暗棋共用）：木頭圓片 + 外緣 + 內圈刻線 + 楷書字；暗棋蓋著的子用深色木背面。
// 尺寸都跟著半徑 r 縮放；陰影沿用 BoardWood 的 PieceShadow，所以 defs 要同時放 WoodDefs 與 XiangqiPieceDefs。

import { PieceShadow } from './BoardWood';

// 楷書優先，沒有就退回系統襯線字；棋子字與河界共用
export const KAI_FONT = '"BiauKai", "DFKai-SB", "KaiTi", "STKaiti", "Kaiti TC", serif';
const INK = { red: '#a8231a', black: '#1f1a14' } as const;

export function XiangqiPieceDefs({ id }: { id: (name: string) => string }) {
    return (
        <>
            <radialGradient id={id('piece')} cx="0.38" cy="0.32" r="0.75">
                <stop offset="0" stopColor="#fbedcc" />
                <stop offset="0.6" stopColor="#ecd29e" />
                <stop offset="1" stopColor="#d2ab68" />
            </radialGradient>
            <radialGradient id={id('piece-back')} cx="0.38" cy="0.32" r="0.75">
                <stop offset="0" stopColor="#b98a52" />
                <stop offset="0.6" stopColor="#94673a" />
                <stop offset="1" stopColor="#6e4824" />
            </radialGradient>
        </>
    );
}

/// 翻開的子。lift 是浮起高度（選中、拖曳）
export function XiangqiPiece({ id, x, y, r, side, char, lift = 0 }: {
    id: (name: string) => string; x: number; y: number; r: number;
    side: 'red' | 'black'; char: string; lift?: number;
}) {
    return (
        <>
            <PieceShadow id={id} x={x} y={y} r={r} lift={lift} />
            <g transform={lift ? `translate(0 ${-lift})` : undefined}>
                <circle cx={x} cy={y} r={r} fill={`url(#${id('piece')})`} stroke="#8a6232" strokeWidth={1.5} />
                <circle cx={x} cy={y} r={r * 0.83} fill="none" stroke={INK[side]} strokeOpacity={0.7} strokeWidth={1.3} />
                <text x={x} y={y + 1} textAnchor="middle" dominantBaseline="central" fill={INK[side]}
                    style={{ fontSize: r * 1.11, fontWeight: 700, fontFamily: KAI_FONT }}>
                    {char}
                </text>
            </g>
        </>
    );
}

/// 蓋著的子（暗棋）：深色木背面，只有刻線沒有字
export function XiangqiPieceBack({ id, x, y, r }: { id: (name: string) => string; x: number; y: number; r: number }) {
    return (
        <>
            <PieceShadow id={id} x={x} y={y} r={r} />
            <circle cx={x} cy={y} r={r} fill={`url(#${id('piece-back')})`} stroke="#4f3216" strokeWidth={1.5} />
            <circle cx={x} cy={y} r={r * 0.78} fill="none" stroke="#f3d9a8" strokeOpacity={0.35} strokeWidth={1.5} />
            <circle cx={x} cy={y} r={r * 0.62} fill="none" stroke="#f3d9a8" strokeOpacity={0.2} strokeWidth={1} />
        </>
    );
}
