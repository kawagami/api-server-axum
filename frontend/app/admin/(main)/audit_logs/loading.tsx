import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <ListTableSkeleton headers={['時間', '管理員', '方法', '路徑', 'Query', '狀態']} rows={10} />;
}
