// 政府電子採購網標案（後端 gov_tenders）
export interface GovTender {
    id: number;
    filename: string;
    date: string; // YYYY-MM-DD
    tender_type: string;
    title: string;
    category: string | null;
    unit_id: string;
    unit_name: string;
    job_number: string;
    companies: string[];
    keyword: string;
    detail_url: string;
    notified_at: string | null;
    created_at: string;
}
