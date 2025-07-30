import { shortenAddress } from "@/lib/utils";
import type { Proposal } from "@voting/core";
import {
  CheckIcon,
  XIcon,
} from "lucide-react";
import { useMemo } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";
import { Link } from "@tanstack/react-router";
import { StatusIcon } from "./status-badge";

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
    // TODO: if active, time left, else, time passed
    return new Date(proposal.createdAt).toLocaleDateString();
  }, [proposal.createdAt]);

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
          votes · {proposal.createdAt.toLocaleDateString()}
        </div>
      </Link>

      {proposal.status === "active" ? (
        <div className="grid grid-cols-2 gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <button className="h-10 w-10 rounded-full border flex items-center justify-center text-green-400 border-green-500">
                <CheckIcon className="h-5 w-5" />
              </button>
            </TooltipTrigger>
            <TooltipContent>
              <p>For</p>
            </TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <button className="h-10 w-10 rounded-full border flex items-center justify-center text-red-400 border-red-400">
                <XIcon className="h-5 w-5" />
              </button>
            </TooltipTrigger>
            <TooltipContent>
              <p>Against</p>
            </TooltipContent>
          </Tooltip>
        </div>
      ) : (
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
