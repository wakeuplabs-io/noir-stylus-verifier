import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "./ui/dialog";
import { WalletIcon } from "lucide-react";
import { useConnect } from "wagmi";

export const ConnectWallet: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const { connect, connectors, isPending: isConnecting } = useConnect();

  return (
    <Dialog>
      {children}

      <DialogContent
        className="w-[360px] rounded-3xl p-4 gap-0 pb-10"
        showCloseButton
      >
        <DialogTitle className="text-center font-medium text-base mb-6">
          Connect your wallet
        </DialogTitle>

        <DialogDescription className="text-sm text-muted-foreground text-center mb-6">
          {isConnecting
            ? "Connecting..."
            : "Connect your wallet to start playing."}
        </DialogDescription>

        {!isConnecting && (
          <div className="flex flex-col gap-2">
            {connectors.map((connector) => (
              <button
                key={connector.id}
                disabled={isConnecting}
                onClick={() => connect({ connector })}
                className="border rounded-xl px-4 py-3 flex items-center gap-2 w-full cursor-pointer hover:bg-muted transition-colors duration-300"
              >
                {connector?.icon ? (
                  <img
                    src={connector?.icon}
                    alt={connector?.name}
                    className="w-[18px] h-[18px]"
                  />
                ) : (
                  <div className="w-[18px] h-[18px] flex items-center justify-center bg-muted rounded-lg">
                    <WalletIcon className="w-4 h-4" />
                  </div>
                )}
                <span className="text-sm">
                  {isConnecting ? "Connecting..." : connector.name}
                </span>
              </button>
            ))}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
};
