import { Account } from "@/components/account";
import { BackButton } from "@/components/back-button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { shortenAddress } from "@/lib/utils";
import { useProposal } from "@/lib/queries/proposal";
import { createFileRoute } from "@tanstack/react-router";
import {
  CheckIcon,
  ClockIcon,
  HelpCircleIcon,
  MousePointer2Icon,
  MousePointerClick,
  MousePointerClickIcon,
  SquareChartGanttIcon,
  XIcon,
} from "lucide-react";
import Markdown from "react-markdown";
import { StatusBadge } from "@/components/status-badge";

export const Route = createFileRoute("/proposals/$id")({
  component: Index,
});

function Index() {
  const { id } = Route.useParams();
  const { data: proposal, isLoading } = useProposal(Number(id));

  if (isLoading || !proposal) return <div>Loading...</div>;
  return (
    <div className="">
      <div className="flex border-b items-center justify-between h-[72px] px-6">
        <BackButton to="/" />

        <Account />
      </div>

      <div className="flex divide-x">
        <div className="p-6 pb-20">
          <h1 className="text-[40px] leading-[1.1em] break-words font-bold mb-4">
            {proposal.metadata.title}
          </h1>

          <StatusBadge status={proposal.status} />

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

            <div>
              <div className="flex items-center gap-2 mb-3 text-muted-foreground text-sm">
                <span>Voting Power: 0</span>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <HelpCircleIcon className="w-4 h-4" />
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>
                      If 0 you are not part of the voters array or you have
                      already voted. Don't worry, only you can see this. If 1
                      means you still can vote!
                    </p>
                  </TooltipContent>
                </Tooltip>
              </div>

              <div className="flex items-center gap-2">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <button className="h-12 w-12 rounded-full border flex items-center justify-center text-green-400 border-green-500">
                      <CheckIcon className="h-5 w-5" />
                    </button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>For</p>
                  </TooltipContent>
                </Tooltip>

                <Tooltip>
                  <TooltipTrigger asChild>
                    <button className="h-12 w-12 rounded-full border flex items-center justify-center text-red-400 border-red-400">
                      <XIcon className="h-5 w-5" />
                    </button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>Against</p>
                  </TooltipContent>
                </Tooltip>
              </div>
            </div>
          </div>

          {/* Results */}
          <div className="mb-6">
            <div className="flex items-center gap-2 mb-3 text-gray-600">
              <SquareChartGanttIcon className="h-4 w-4" />

              <span className="uppercase font-semibold text-sm">Results</span>
            </div>

            <div className="space-y-2">
              <div className="text-gray-900 border border-green-500 py-3 px-4 flex gap-2 rounded-md items-center bg-green-50 font-medium">
                <div className="flex items-center justify-center w-4 h-4 rounded-full bg-green-600 text-white shrink-0">
                  <CheckIcon className="w-3 h-3" />
                </div>
                <span className="w-full ">For</span>
                <span className="">{proposal.for}</span>
                <span className="">50%</span>
              </div>

              <div className="text-gray-900 border border-red-500 py-3 px-4 flex gap-2 rounded-md items-center font-medium">
                <div className="flex items-center justify-center w-4 h-4 rounded-full bg-red-600 text-white shrink-0">
                  <XIcon className="w-3 h-3" />
                </div>
                <span className="w-full ">Against</span>
                <span className="">{proposal.against}</span>
                <span className="">50%</span>
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
                    <div>{proposal.createdAt.toDateString()}</div>
                  </div>
                </div>

                <div className="mb-3 last:mb-0 h-[44px]">
                  <h4 className=" font-medium">End</h4>
                  <div className="flex gap-2 items-center text-muted-foreground">
                    <div>{proposal.deadline.toDateString()}</div>
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
