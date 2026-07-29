import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return (
        <ListTableSkeleton
            headers={['語言', '表記', '讀音', '釋義', '詞性', '難度', '✗/✓', '狀態', '操作']}
            rows={10}
        />
    );
}
