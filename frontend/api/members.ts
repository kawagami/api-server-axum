"use server";

import adminRequest from "@/libs/adminRequest";
import memberRequest from "@/libs/memberRequest";
import type { Member, MemberDetail } from "@/types";

export async function getMembers(): Promise<Member[]> {
    return adminRequest<Member[]>({
        url: `${process.env.API_URL}/members`,
    });
}

export async function getMember(id: number): Promise<MemberDetail> {
    return adminRequest<MemberDetail>({
        url: `${process.env.API_URL}/members/${id}`,
    });
}

export async function getCurrentMember(): Promise<MemberDetail> {
    return memberRequest<MemberDetail>({ url: `${process.env.API_URL}/members/me` });
}
