// 訪客留言(對應後端 backend/src/structs/messages.rs 的 Message / NewMessage)
export interface ContactMessage {
    id: number;
    name: string | null;
    email: string | null;
    content: string;
    created_at: string;
}

export interface NewContactMessage {
    name?: string;
    email?: string;
    content: string;
}
