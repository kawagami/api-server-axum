import { BorderedTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <BorderedTableSkeleton headers={['ID', '名稱', 'Email', '註冊時間']} rows={8} />;
}
