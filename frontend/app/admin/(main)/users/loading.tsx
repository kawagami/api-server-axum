import { BorderedTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <BorderedTableSkeleton headers={['ID', '名稱', 'Email', '角色', '操作']} rows={6} />;
}
