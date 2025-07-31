import { AudioWaveform, CheckIcon, WalletIcon } from "lucide-react";
import { XIcon } from "lucide-react";
import { Dialog, DialogClose, DialogContent } from "./ui/dialog";
import { useCastVote } from "@/hooks/proposal";
import { useAccount as useEvmAccount } from "wagmi";
import { Tooltip } from "react-tooltip";

export const CastVote: React.FC<{
  proposalId: number;
}> = ({ proposalId }) => {
  const { connector } = useEvmAccount();
  const { castVote, disabled, isGeneratingProof, isSendingTransaction } =
    useCastVote(proposalId);

  return (
    <>
      <div className="flex items-center gap-2">
        <button
          disabled={disabled || connector === undefined}
          onClick={() => castVote(true)}
          className="h-12 w-12 rounded-full border flex items-center justify-center text-success border-success disabled:opacity-50 disabled:cursor-not-allowed"
          data-tooltip-id="for-tooltip"
        >
          <CheckIcon className="h-5 w-5" />
        </button>
        <Tooltip id="for-tooltip" content="For" />

        <button
          disabled={disabled || connector === undefined}
          onClick={() => castVote(false)}
          className="h-12 w-12 rounded-full border flex items-center justify-center text-danger border-danger disabled:opacity-50 disabled:cursor-not-allowed"
          data-tooltip-id="against-tooltip"
        >
          <XIcon className="h-5 w-5" />
        </button>
        <Tooltip id="against-tooltip" content="Against" />
      </div>

      <Dialog open={isGeneratingProof || isSendingTransaction}>
        <DialogContent className="w-[360px] rounded-3xl p-4 gap-0 pb-10">
          <div className=" mb-4 w-full flex justify-end">
            <DialogClose className="ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 rounded-full bg-muted h-4 w-4 flex items-center justify-center">
              <XIcon />
              <span className="sr-only">Close</span>
            </DialogClose>
          </div>

          {isGeneratingProof && (
            <div className="flex flex-col items-center">
              <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                <AudioWaveform className="w-4 h-4" />
              </div>

              <div className="font-medium text-center mb-1">
                Generating proof...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                This may take a few seconds.
              </div>
            </div>
          )}

          {isSendingTransaction && (
            <div className="flex flex-col items-center">
              {connector?.icon ? (
                <img
                  src={connector?.icon}
                  alt={connector?.name}
                  className="w-10 h-10 mb-4"
                />
              ) : (
                <div className="w-10 h-10 mb-4 flex items-center justify-center bg-muted rounded-full">
                  <WalletIcon className="w-4 h-4" />
                </div>
              )}

              <div className="font-medium text-center mb-1">
                Waiting for {connector?.name ?? "wallet"}...
              </div>

              <div className="text-xs text-muted-foreground text-center">
                Don’t see your wallet? Check your other browser windows.
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
};
