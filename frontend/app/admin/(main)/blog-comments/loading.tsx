import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <ListTableSkeleton headers={['時間', '來自', '內容', '文章', '']} rows={10} />;
}
