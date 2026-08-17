import { ListTableSkeleton } from '@/components/loading/table-skeleton';
import { SKELETON_HEADERS } from './skeleton-headers';

export default function Loading() {
    return <ListTableSkeleton headers={SKELETON_HEADERS} rows={10} />;
}
