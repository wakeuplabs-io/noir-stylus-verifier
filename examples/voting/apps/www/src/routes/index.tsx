import { Account } from "@/components/account";
import { ProposalCard } from "@/components/proposal-card";
import { useProposals } from "@/lib/queries/proposal";
import { createFileRoute, Link } from "@tanstack/react-router";
import { PenBoxIcon, SearchIcon } from "lucide-react";
import { useEffect, useMemo, useRef } from "react";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
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
    <div className="">
      <div className="flex border-b items-center justify-between h-[72px] px-6">
        <div className="flex items-center gap-4 w-full">
          <SearchIcon className="w-4 h-4" />
          <input
            type="text"
            placeholder="Search for a proposal"
            className="w-full h-full outline-none"
          />
        </div>

        <Account />
      </div>

      <div className="p-6 flex items-center justify-between">
        <div className="relative border rounded-full h-[46px] px-4 flex items-center min-w-20">
          <span className="absolute left-3 top-0 translate-y-[-50%] text-xs text-gray-500 bg-white px-1">
            Status
          </span>
          <span className="text">Any</span>
        </div>

        <Link
          to="/proposals/new"
          className="flex items-center gap-2 rounded-full border h-[46px] w-[46px] shrink-0 justify-center"
        >
          <PenBoxIcon className="h-4 w-4" />
        </Link>
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
