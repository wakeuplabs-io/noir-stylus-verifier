export type ProposalStatus = "active" | "passed" | "rejected";

export type Proposal = {
    id: string;
    title: string;
    author: string;
    for: number;
    against: number;
    abstain: number;
    status: ProposalStatus;
    createdAt: Date;
}