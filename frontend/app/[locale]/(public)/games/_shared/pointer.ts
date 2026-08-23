// 指標座標 → SVG viewBox 座標。拖曳落子要靠它把 clientX/Y 換回盤面格。
// 盤面元件用的是 viewBox 座標系（與實際像素大小無關，因為 svg 會被縮放）。
export function toViewBox(
    e: { clientX: number; clientY: number },
    svg: SVGSVGElement,
    vbWidth: number,
    vbHeight: number,
): [number, number] {
    const r = svg.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) return [-1, -1];
    return [
        ((e.clientX - r.left) * vbWidth) / r.width,
        ((e.clientY - r.top) * vbHeight) / r.height,
    ];
}

// 觸控裝置（含觸控筆）判定：用來決定「點一下就落子」還是「點一下先預覽、再點確認」。
// 依事件的 pointerType 判斷而非裝置能力 —— 二合一筆電接滑鼠時不該被當成觸控。
export function isTouchPointer(e: { pointerType?: string }): boolean {
    return e.pointerType === 'touch' || e.pointerType === 'pen';
}
