import { ImageResponse } from "next/og";

/**
 * 全站預設的社群分享圖（各頁沒有自己的 opengraph-image 時就用這張）。
 * 只用拉丁字母：ImageResponse 的預設字型沒有中日韓字，中文會變成方框。
 * 色值寫死 forest（預設主題）的 primary 色階——這裡讀不到 CSS variable。
 */
export const alt = "Kawa's Homes";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default async function OpengraphImage() {
    return new ImageResponse(
        (
            <div
                style={{
                    width: "100%",
                    height: "100%",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: 24,
                    background: "linear-gradient(160deg, #081c15 0%, #1b4332 55%, #2d6a4f 100%)",
                    color: "#d8f3dc",
                    fontFamily: "sans-serif",
                }}
            >
                <div style={{ fontSize: 92, fontWeight: 700, letterSpacing: -2 }}>Kawa&apos;s Homes</div>
                <div style={{ fontSize: 36, color: "#95d5b2" }}>Blog · Tools · Games</div>
                <div style={{ fontSize: 26, color: "#74c69d" }}>Rust · Axum · Next.js</div>
            </div>
        ),
        size,
    );
}
