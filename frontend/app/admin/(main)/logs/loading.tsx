import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <ListTableSkeleton headers={['ID', '層級', '訊息', '來源模組', '檔案', '時間']} rows={10} />;
}
