"use server";

import adminRequest from "@/libs/adminRequest";
import memberRequest from "@/libs/memberRequest";
import type { Member, MemberDetail, PaginatedResponse } from "@/types";

export async function getMembers(): Promise<PaginatedResponse<Member>> {
    const res = await adminRequest<PaginatedResponse<Member>>({
        url: `${process.env.API_URL}/members`,
    });
    return res ?? { data: [], total: 0 };
}

export async function getMember(id: number): Promise<MemberDetail> {
    return adminRequest<MemberDetail>({
        url: `${process.env.API_URL}/members/${id}`,
    });
}

export async function getCurrentMember(): Promise<MemberDetail> {
    return memberRequest<MemberDetail>({ url: `${process.env.API_URL}/members/me` });
}
