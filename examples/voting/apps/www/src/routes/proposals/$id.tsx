import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { shortenAddress } from "@/lib/utils";
import { createFileRoute, Link } from "@tanstack/react-router";
import {
  ArrowLeftIcon,
  ChartPieIcon,
  CheckIcon,
  HelpCircleIcon,
  MinusIcon,
  MousePointer2,
  MousePointer2Icon,
  RadioIcon,
  SquareChartGantt,
  SquareChartGanttIcon,
  XIcon,
} from "lucide-react";
import Markdown from "react-markdown";

const MOCK_MD = `### Non-Constitutional

## Key Changes Made on July 23, 2025

The 2025 Events Budget will no longer be dissolved, and instead funds will be moved to a yield bearing account managed by the Arbitrum Foundation/treasury managers. The Events Budget process will remain as currently structured until the end of 2025. 

## Abstract

* Entropy proposes transferring the DAO’s 2025 Events Budget (~1.04M USDC) to the [ATMC](https://snapshot.box/#/s:arbitrumfoundation.eth/proposal/0xfab795313df4ef1023c5f7f9027857637cc3841d62dc0b54796fbfa5f8096919), which will top-up the budget allocated to onchain treasury managers focused on stablecoin strategies. The Events Budget will continue to operate as designed through the end of 2025, but instead be earning yield as it sits idle.
* We also propose to send the ~1.5M USDC left over from the ARDC v2, which was recently denied an extension, to be sent to the ATMC to further top-up the budget allocation to onchain treasury managers. This modifies the recently passed proposal which defined the next step as the “AF returning all unused funds to the treasury”, which would forfeit yield until another onchain proposal process in its entirety is carried out.
* Lastly, we propose sending any remaining funds from the ADPC Security Subsidies budget to the ATMC, which will again be used to top-up the onchain treasury managers budget.
* The proposal will be moved to a vote on July 24th, and if passed, funds from the ARDC V2 and 2025 Events Budget will be transferred within 7 days from their current addresses to this [address](https://arbiscan.io/address/0xd8e35e2450003cd8d50cc804aee4db0a8872b7a9#tokentxns) where the ~4.95M USDC for the ATMC currently resides. Following the completion of the ADPC Security Subsidies, whatever USDC remains will also be sent to the same address designated for onchain stablecoin strategies managed by DAO approved treasury managers.

## Motivation & Rationale

### 2025 Events Budget

While Entropy believes events are a vertical that should for the most part fall under the Foundation’s scope and that smaller events should be funded by the D.A.O. grants program, the community has requested that this issue be taken up separately rather than being consolidated into a larger Snapshot vote. As such, we have modified this proposal to simply move the 2025 Events Budget into a yield bearing account with the ATMC to top up the stablecoin strategy allocation to treasury managers.

The Events Budget remains intact and will continue to follow the process outlined per the original [proposal](https://forum.arbitrum.foundation/t/establishing-a-dao-events-budget-for-2025/26734) through 2025. Once 2025 comes to an end, the Stablecoins will remain allocated to the ATMC rather than being returned to the DAO treasury to ensure they continue to earn yield. 

### ARDC V2

The ARDC v2 Extension [proposal](https://snapshot.box/#/s:arbitrumfoundation.eth/proposal/0x765b297702f21736e7b52c23d781dc76968519fef9c7b1ca1f924d167b2a6899) recently came to a close, which resulted in the collective NOT being extended. The instructions per the proposal were defined as “(The) AF returning all unused funds to the treasury”. However, now that the funds have already been converted from ARB into stablecoins, there is no sense in sending these funds back to the treasury to sit idle, and forcing another proposer to go through the entire onchain governance process to ensure they are earning yield for the DAO.

### ADPC Security Subsidies

670,543.5 USDC was transferred from the MSS to an Arbitrum Foundation controlled [address](https://arbiscan.io/address/0x073a8f65ed92da9f9f64db0d69f424aa978e1ccd#tokentxns) as a part of the MSS wind down. Entropy Advisors is engaged in ongoing discussions with the Arbitrum Foundation in regards to what portion of these funds have already been committed to service providers versus what is remaining/owed to the DAO. We propose sending any remaining ADPC Security Subsidy funds to the ATMC as well once the AF pays out any outstanding contractual obligations. We do not want this process to stall this proposal, as doing so would potentially result in the ARDC v2 funds being returned to the DAO treasury. Instead, we seek the DAO’s approval to reallocate these funds to the treasury managers deploying onchain stablecoin strategies once the situation has been sorted.

## Specifications

The 2025 Events Budget currently holds [1,044,095.59 USDC](https://arbiscan.io/address/0x1a5184b21a8731e60a98511edee1c917744267c0). If approved, the entirety of that balance will be moved to the Foundation controlled [wallet](https://arbiscan.io/address/0xd8e35e2450003cd8d50cc804aee4db0a8872b7a9#tokentxns) that is designated for onchain stablecoin strategies managed by DAO approved treasury managers. The Events Budget process will remain unchanged for authors who wish to pull from this allocation of stablecoins to host an event.

The ARDC V2 currently holds [1,503,604.08 USDC](https://arbiscan.io/address/0x22a017676274d67dec950391569b3a13d5199a4a) and 112,245.95 ARB. The ARDC has finished and posted its final deliverable, the Arbitrum Ecosystem Mapping Report. Final invoices will now be obtained from the two research service providers, Castle Capital and DeFiLlama, and USDC payments initiated with the Arbitrum Foundation. Once these payments have been sent, the remaining USDC balance will be moved by the Foundation to the wallet designated for onchain stablecoin strategies. The remaining ARB will be returned to the DAO’s treasury.

As mentioned above, once the Arbitrum Foundation pays out any outstanding contractual obligations related to the ADPC Security Subsidies program, the remaining USDC will be sent to the ATMC. The ATMC will update the DAO once this has occurred on its dedicated forum thread.

The 15M ARB allocated to stablecoin strategies was recently converted into ~4.95M USDC, but this proposal would increase this allocation to ~7.5-8M USDC. These funds will either be split pro-rata amongst various treasury managers already approved by the DAO, or allocated according to the newly introduced ATMC procedures i.e., allocation recommendation from Entropy followed by OAT approval. These funds have been idle for several months, which could have otherwise been earning yield. This proposal is a part of a larger effort to strengthen the Arbitrum DAO’s financial position to enable long-term sustainable growth/increasing revenue.

## Timeline

July 14th: Forum post
July 24th - July 31st : Snapshot vote
By August 7th: If passed, funds will be transferred to the stablecoin strategy address within 7 days`;

export const Route = createFileRoute("/proposals/$id")({
  component: Index,
});

function Index() {
  return (
    <div className="">
      <div className="max-w-5xl mx-auto border-x min-h-screen w-full">
        <div className="flex border-b items-center justify-between h-[72px] px-6">
          <Link to="/" className="flex items-center gap-4 w-full">
            <ArrowLeftIcon className="w-4 h-4" />
          </Link>

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

        <div className="flex divide-x">
          <div className="p-6 pb-20">
            <h1 className="text-[40px] leading-[1.1em] break-words font-bold mb-4">
              Consolidate Idle USDC to the ATMC’s Stablecoin Balance
            </h1>

            <div className="flex items-center gap-2 rounded-full mb-6 text-white bg-green-500 px-2 pr-3 py-1 w-min">
              <RadioIcon className="w-4 h-4" />
              <span className="text-sm">Active</span>
            </div>

            <div className="flex items-center gap-2 py-4 mb-6">
              <img
                src="https://cdn.stamp.fyi/avatar/0x9D39B627E6769B0b77f03825C118Ec48c84A8fbD?s=36"
                alt="avatar"
                className="w-[32px] h-[32px] rounded-full"
              />
              <div>
                <div className="text-sm">
                  {shortenAddress("0x9D39B627E6769B0b77f03825C118Ec48c84A8fbD")}
                </div>
                <div className="text-xs text-muted-foreground space-x-2">
                  5 days ago · #1
                </div>
              </div>
            </div>

            <div className="prose">
              <Markdown>{MOCK_MD}</Markdown>
            </div>
          </div>

          <div className="w-[600px] p-6 pb-20">
            {/* Cast your vote */}
            <div className="mb-6">
              <div className="flex items-center gap-2 mb-3 text-gray-800">
                <MousePointer2Icon className="h-4 w-4" />

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

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <button className="h-12 w-12 rounded-full border flex items-center justify-center text-gray-400 border-gray-400">
                        <MinusIcon className="h-5 w-5" />
                      </button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Abstain</p>
                    </TooltipContent>
                  </Tooltip>
                </div>
              </div>
            </div>

            {/* Results */}
            <div>
              <div className="flex items-center gap-2 mb-3 text-gray-800">
                <SquareChartGanttIcon className="h-4 w-4" />

                <span className="uppercase font-semibold text-sm">Results</span>
              </div>

              <div className="space-y-2">
                <div className="text-gray-900 border border-green-500 py-3 px-4 flex gap-2 rounded-md items-center bg-green-50 font-medium">
                  <div className="flex items-center justify-center w-4 h-4 rounded-full bg-green-600 text-white shrink-0">
                    <CheckIcon className="w-3 h-3" />
                  </div>
                  <span className="w-full ">For</span>
                  <span className="">500</span>
                  <span className="">50%</span>
                </div>

                <div className="text-gray-900 border border-red-500 py-3 px-4 flex gap-2 rounded-md items-center font-medium">
                  <div className="flex items-center justify-center w-4 h-4 rounded-full bg-red-600 text-white shrink-0">
                    <XIcon className="w-3 h-3" />
                  </div>
                  <span className="w-full ">Against</span>
                  <span className="">500</span>
                  <span className="">50%</span>
                </div>

                <div className="text-gray-900 border border-gray-500 py-3 px-4 flex gap-2 rounded-md items-center font-medium">
                  <div className="flex items-center justify-center w-4 h-4 rounded-full bg-gray-600 text-white shrink-0">
                    <MinusIcon className="w-3 h-3" />
                  </div>
                  <span className="w-full ">Abstain</span>
                  <span className="">500</span>
                  <span className="">50%</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
