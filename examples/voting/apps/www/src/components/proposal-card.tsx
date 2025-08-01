import { shortenAddress } from "@/lib/utils";
import type { Proposal } from "@voting/core";
import { useMemo } from "react";
import { Link } from "@tanstack/react-router";
import { StatusIcon } from "./status-badge";
import { formatDistanceToNow } from "date-fns";

export const ProposalCard: React.FC<{
  proposal: Proposal;
}> = ({ proposal }) => {
  const totalVotes = proposal.for + proposal.against;

  const forPercentage = useMemo(() => {
    return (proposal.for / totalVotes) * 100;
  }, [proposal.for, totalVotes]);

  const againstPercentage = useMemo(() => {
    return (proposal.against / totalVotes) * 100;
  }, [proposal.against, totalVotes]);

  const timeString = useMemo(() => {
    if (proposal.status === "active") {
      return formatDistanceToNow(new Date(proposal.deadline)) + " left";
    }
    return formatDistanceToNow(proposal.deadline) + " ago";
  }, [proposal.deadline]);

  return (
    <div className="flex items-center justify-between w-full gap-2 py-[14px]">
      <Link to="/proposals/$id" params={{ id: proposal.id.toString() }}>
        <div className="flex items-center gap-2">
          <StatusIcon status={proposal.status} />
          <span className="text-lg font-bold my-1">
            {proposal.metadata.title}
          </span>
        </div>

        <div className="text-muted-foreground text-sm">
          #{proposal.id} by {shortenAddress(proposal.author)} · {totalVotes}{" "}
          votes · {timeString}
        </div>
      </Link>

      {proposal.status !== "active" && (
        <div>
          <div className="w-[100px] h-[6px] rounded-full flex overflow-hidden">
            <div
              className="h-full bg-green-500"
              style={{ width: `${forPercentage}%` }}
            ></div>
            <div
              className="h-full bg-red-400"
              style={{ width: `${againstPercentage}%` }}
            ></div>
          </div>
        </div>
      )}
    </div>
  );
};
