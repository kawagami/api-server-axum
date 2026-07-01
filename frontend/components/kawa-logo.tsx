// 站名字標（inline SVG 以便吃 CSS variables，跟著 data-theme 變色）
// 圖形說明：monoline "Kawa" + 尾端葉子 + 底下河流波紋
export default function KawaLogo({ width = 100, height = 40 }: { width?: number; height?: number }) {
    return (
        <svg width={width} height={height} viewBox="0 0 240 96" role="img" aria-label="Kawa">
            {/* 字母 K a w a（monoline，圓頭筆畫） */}
            <g
                fill="none"
                style={{ stroke: 'rgb(var(--primary-600))' }}
                strokeWidth="11"
                strokeLinecap="round"
                strokeLinejoin="round"
            >
                <path d="M20 12 V64" />
                <path d="M48 14 L24 40" />
                <path d="M30 34 L50 64" />
                <circle cx="78" cy="49" r="15" />
                <path d="M93 36 V64" />
                <path d="M106 34 L115 64 L124 44 L133 64 L142 34" />
                <circle cx="170" cy="49" r="15" />
                <path d="M185 36 V64" />
            </g>
            {/* 從最後一筆冒出的葉子 */}
            <path d="M188 31 C 194 14, 212 8, 224 13 C 222 28, 204 38, 190 33 Z" style={{ fill: 'rgb(var(--primary-400))' }} />
            <path d="M191 31 C 200 25, 210 19, 220 15" fill="none" style={{ stroke: 'rgb(var(--primary-700))' }} strokeWidth="2.5" strokeLinecap="round" />
            {/* 河流波紋 */}
            <path d="M16 82 Q 30 74, 44 82 T 72 82 T 100 82 T 128 82 T 156 82 T 184 82"
                fill="none" style={{ stroke: 'rgb(var(--primary-300))' }} strokeWidth="6" strokeLinecap="round" />
            <path d="M40 90 Q 54 84, 68 90 T 96 90 T 124 90 T 152 90"
                fill="none" style={{ stroke: 'rgb(var(--primary-200))' }} strokeWidth="4" strokeLinecap="round" />
        </svg>
    );
}
