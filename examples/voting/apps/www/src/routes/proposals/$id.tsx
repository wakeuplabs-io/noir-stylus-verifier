import { AccountManager } from "@/components/account-manager";
import { BackButton } from "@/components/back-button";
import { cn, shortenAddress } from "@/lib/utils";
import { useProposal } from "@/hooks/proposal";
import { createFileRoute } from "@tanstack/react-router";
import {
  CheckIcon,
  ClockIcon,
  HelpCircleIcon,
  MousePointerClickIcon,
  SquareChartGanttIcon,
  XIcon,
} from "lucide-react";
import Markdown from "react-markdown";
import { StatusBadge } from "@/components/status-badge";
import { useEffect, useMemo } from "react";
import { CastVote } from "@/components/cast-vote";
import { Tooltip } from "react-tooltip";
import { useZkAccount } from "@/hooks/account";

export const Route = createFileRoute("/proposals/$id")({
  component: Index,
});

function Index() {
  const { id } = Route.useParams();
  const { account } = useZkAccount();
  const { data: { proposal, isEligible, alreadyVoted } = {}, isPending } =
    useProposal(Number(id));

  const voteCount = useMemo(() => {
    if (!proposal) return 0;
    return proposal.for + proposal.against;
  }, [proposal]);

  const forPercentage = useMemo(() => {
    if (!proposal) return 0;
    return (proposal.for / (proposal.for + proposal.against)) * 100;
  }, [proposal]);

  const againstPercentage = useMemo(() => {
    if (!proposal) return 0;
    return (proposal.against / (proposal.for + proposal.against)) * 100;
  }, [proposal]);

  if (isPending || !proposal)
    return (
      <div className="flex items-center justify-center h-full w-full p-10">
        Loading...
      </div>
    );

  return (
    <div>
      <div className="flex border-b items-center justify-between h-[72px] px-6">
        <BackButton to="/" />

        <AccountManager />
      </div>

      <div className="flex divide-x">
        <div className="p-6 pb-20 w-full">
          <h1 className="text-[40px] leading-[1.1em] break-words font-bold mb-4">
            {proposal.metadata.title}
          </h1>

          <StatusBadge status={proposal.status} />

          {/* Author */}
          <div className="flex items-center gap-2 py-4 mb-6">
            <img
              src="/avatar.webp"
              alt="avatar"
              className="w-[32px] h-[32px] rounded-full"
            />
            <div>
              <div className="text-sm">{shortenAddress(proposal.author)}</div>
              <div className="text-xs text-muted-foreground space-x-2">
                {proposal.createdAt.toLocaleDateString()} · #{proposal.id}
              </div>
            </div>
          </div>

          <div className="prose">
            <Markdown>{proposal.metadata.body}</Markdown>
          </div>
        </div>

        <div className="w-[600px] p-6 pb-20">
          {/* Cast your vote */}
          <div className="mb-6">
            <div className="flex items-center gap-2 mb-3 text-gray-600">
              <MousePointerClickIcon className="h-4 w-4" />

              <span className="uppercase font-semibold text-sm">
                Cast your vote
              </span>
            </div>

            {account ? (
              <div className="text-muted-foreground text-sm">
                <div className="flex items-center gap-2 mb-3 text-muted-foreground text-sm">
                  <span>
                    Available votes: {isEligible && !alreadyVoted ? 1 : 0}
                  </span>

                  <HelpCircleIcon
                    className="w-4 h-4"
                    data-tooltip-id="vote-tooltip"
                  />
                  <Tooltip
                    id="vote-tooltip"
                    content={
                      proposal.status === "active" ? (
                      !isEligible
                        ? "You are not part of the voters array."
                        : alreadyVoted
                        ? "You have already voted."
                        : "You have 1 vote left."
                      ) : (
                        "This proposal is not active."
                      )
                    }
                  />
                </div>

                <CastVote proposalId={Number(id)} />
              </div>
            ) : (
              <div className="text-muted-foreground text-sm">
                Connect wallet to cast your vote
              </div>
            )}
          </div>

          {/* Results */}
          <div className="mb-6">
            <div className="flex items-center gap-2 mb-3 text-gray-600">
              <SquareChartGanttIcon className="h-4 w-4" />

              <span className="uppercase font-semibold text-sm">Results</span>
            </div>

            <div
              className={cn(
                "flex flex-col gap-2",
                proposal.for < proposal.against && "flex-col-reverse"
              )}
            >
              <div
                className={cn(
                  "text-gray-900 border border-success py-3 px-4 flex gap-2 rounded-md items-center font-medium",
                  proposal.for > proposal.against && "bg-success/10"
                )}
              >
                <div className="flex items-center justify-center w-4 h-4 rounded-full bg-success text-white shrink-0">
                  <CheckIcon className="w-3 h-3" />
                </div>
                <span className="w-full ">For</span>
                <span className="text-sm shrink-0 font-semibold">
                  {proposal.for} - {voteCount > 0 && forPercentage.toFixed(2)}%
                </span>
              </div>

              <div
                className={cn(
                  "text-gray-900 border border-danger py-3 px-4 flex gap-2 rounded-md items-center font-medium",
                  proposal.for < proposal.against && "bg-danger/10"
                )}
              >
                <div className="flex items-center justify-center w-4 h-4 rounded-full bg-danger text-white shrink-0">
                  <XIcon className="w-3 h-3" />
                </div>
                <span className="w-full ">Against</span>
                <span className="shrink-0 text-sm font-semibold">
                  {proposal.against} -{" "}
                  {voteCount > 0 && againstPercentage.toFixed(2)}%
                </span>
              </div>
            </div>
          </div>

          {/* Timeline */}
          <div>
            <div className="flex items-center gap-2 mb-3 text-gray-600">
              <ClockIcon className="h-4 w-4" />

              <span className="uppercase font-semibold text-sm">Timeline</span>
            </div>

            <div className="flex">
              <div className="mt-1 ml-2">
                <div className="flex relative h-[60px] last:h-0">
                  <div className="absolute size-[15px] inline-block rounded-full left-[-7px] border-4 border-white bg-gray-900"></div>
                  <div className="border-l pr-4 mt-3 border-gray-900"></div>
                </div>

                <div className="flex relative h-[60px] last:h-0">
                  <div className="absolute size-[15px] inline-block rounded-full left-[-7px] border-4 border-white bg-gray-900"></div>
                </div>
              </div>
              <div className="flex-auto leading-6">
                <div className="mb-3 last:mb-0 h-[44px]">
                  <h4 className="font-medium">Created</h4>
                  <div className="flex gap-2 items-center text-muted-foreground">
                    <div>
                      {proposal.createdAt.toLocaleString(undefined, {
                        dateStyle: "medium",
                        timeStyle: "short",
                      })}
                    </div>
                  </div>
                </div>

                <div className="mb-3 last:mb-0 h-[44px]">
                  <h4 className=" font-medium">End</h4>
                  <div className="flex gap-2 items-center text-muted-foreground">
                    <div>
                      {proposal.deadline.toLocaleString(undefined, {
                        dateStyle: "medium",
                        timeStyle: "short",
                      })}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
