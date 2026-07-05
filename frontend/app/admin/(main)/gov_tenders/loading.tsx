import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <ListTableSkeleton headers={['公告日', '類型', '標案名稱', '機關', '廠商', '關鍵字']} rows={10} />;
}
