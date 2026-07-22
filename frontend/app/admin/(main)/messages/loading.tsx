import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <ListTableSkeleton headers={['時間', '名字', 'Email', '內容', '']} rows={10} />;
}
