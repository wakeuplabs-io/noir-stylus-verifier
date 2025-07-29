import { ProposalCard } from "@/components/proposal-card";
import { shortenAddress } from "@/lib/utils";
import { createFileRoute, Link } from "@tanstack/react-router";
import { PenBoxIcon, SearchIcon } from "lucide-react";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  return (
    <div className="">
      <div className="max-w-5xl mx-auto border-x min-h-screen w-full">
        <div className="flex border-b items-center justify-between h-[72px] px-6">
          <div className="flex items-center gap-4 w-full">
            <SearchIcon className="w-4 h-4" />
            <input
              type="text"
              placeholder="Search for a proposal"
              className="w-full h-full outline-none"
            />
          </div>

          <button className="flex items-center gap-2 rounded-full border h-[46px] px-4 shrink-0">
            <img
              src="https://cdn.stamp.fyi/avatar/0x9D39B627E6769B0b77f03825C118Ec48c84A8fbD?s=36"
              alt="avatar"
              className="w-[18px] h-[18px] rounded-full"
            />
            <span className="text-sm">
              {shortenAddress("0x9D39B627E6769B0b77f03825C118Ec48c84A8fbD")}
            </span>
          </button>
        </div>

        <div className="p-6 flex items-center justify-between">
          <div className="relative border rounded-full h-[46px] px-4 flex items-center min-w-20">
            <span className="absolute left-3 top-0 translate-y-[-50%] text-xs text-gray-500 bg-white px-1">
              Status
            </span>
            <span className="text">Any</span>
          </div>

          <Link to="/proposals/new" className="flex items-center gap-2 rounded-full border h-[46px] w-[46px] shrink-0 justify-center">
            <PenBoxIcon className="h-4 w-4" />
          </Link>
        </div>

        <div>
          <div className="uppercase text-sm font-medium text-muted-foreground px-6 py-2 border-b">
            Proposals
          </div>

          <ul className="divide-y mx-6">
            <ProposalCard
              proposal={{
                id: "1",
                title: "Proposal 1",
                author: "0x9D39B627E6769B0b77f03825C118Ec48c84A8fbD",
                for: 100,
                against: 200,
                abstain: 100,
                status: "active",
                createdAt: new Date(),
              }}
            />
            <ProposalCard
              proposal={{
                id: "1",
                title: "Proposal 2",
                author: "0x9D39B627E6769B0b77f03825C118Ec48c84A8fbD",
                for: 100,
                against: 200,
                abstain: 100,
                status: "passed",
                createdAt: new Date(),
              }}
            />
          </ul>
        </div>
      </div>
    </div>
  );
}
