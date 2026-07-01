import { BorderedTableSkeleton } from "@/components/loading/table-skeleton";

export default function Loading() {
    return <BorderedTableSkeleton headers={['ID', 'Name', 'Email', 'Roles']} rows={6} />;
}
