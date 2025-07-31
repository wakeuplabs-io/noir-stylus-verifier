import { AccountManager } from "@/components/account";
import { ProposalCard } from "@/components/proposal-card";
import { useZkAccount } from "@/hooks/account";
import { useProposals } from "@/hooks/proposal";
import { createFileRoute, Link } from "@tanstack/react-router";
import {  AudioWaveformIcon, PenBoxIcon } from "lucide-react";
import { useEffect, useMemo, useRef } from "react";
import { Tooltip } from "react-tooltip";


export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const { account } = useZkAccount();

  const loadMoreRef = useRef(null);
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage } =
    useProposals();

  const sortedProposals = useMemo(() => {
    return data?.pages.flatMap((page) => page.proposals).sort((a, b) => b.id - a.id) ?? [];
  }, [data]);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasNextPage) {
          fetchNextPage();
        }
      },
      { threshold: 1.0 }
    );

    if (loadMoreRef.current) observer.observe(loadMoreRef.current);
    return () => observer.disconnect();
  }, [fetchNextPage, hasNextPage]);

  return (
    <div>
      <div className="flex border-b items-center justify-between h-[72px] px-6">
        <div className="flex items-center gap-4 w-full">
          <AudioWaveformIcon className="w-4 h-4" />
          <h1 className="text-xl font-bold">ZK Voting Noir Stylus</h1>
        </div>

        <AccountManager />
      </div>

      <div className="p-6 flex items-center justify-end">
        <Link
          to="/proposals/new"
          className="flex items-center gap-2 rounded-full border h-[46px] w-[46px] shrink-0 justify-center"
          disabled={!account}
          data-tooltip-id="new-proposal-tooltip"
        >
          <PenBoxIcon className="h-4 w-4" />
        </Link>
        {!account && <Tooltip id="new-proposal-tooltip" content="Connect wallet to create a new proposal" />}
      </div>

      <div>
        <div className="uppercase text-sm font-medium text-muted-foreground px-6 py-2 border-b">
          Proposals
        </div>

        <div className="divide-y mx-6">
          {sortedProposals.map((proposal, id) => (
            <ProposalCard key={id} proposal={proposal} />
          ))}

          <div ref={loadMoreRef} className="py-4">
            {isFetchingNextPage
              ? "Loading more..."
              : hasNextPage
              ? "Load more"
              : "No more items"}
          </div>
        </div>
      </div>
    </div>
  );
}
