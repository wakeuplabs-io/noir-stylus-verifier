import { useZkAccount } from "@/hooks/account";
import { shortenAddress } from "@/lib/utils";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "./ui/dialog";
import { useEffect, useState } from "react";
import { ArrowLeftIcon, Check, Copy, WalletIcon, XIcon } from "lucide-react";
import {
  useAccount as useEvmAccount,
  useConnect as useEvmConnect,
  useDisconnect as useEvmDisconnect,
  type Connector,
} from "wagmi";
import { Button } from "./ui/button";
import { ACCOUNT_MESSAGE } from "@voting/core";

type Step = "connect" | "sign";

export const AccountManager: React.FC = () => {
  const { address: addressEvm } = useEvmAccount();
  const { disconnect: disconnectEvm } = useEvmDisconnect();
  const {
    connect: connectEvm,
    connectors: connectorsEvm,
    isPending: isConnectingEvm,
  } = useEvmConnect();
  const {
    connect: connectZk,
    isPending: isConnectingZk,
    account: accountZk,
    disconnect: disconnectZk,
  } = useZkAccount();

  const [isOpen, setIsOpen] = useState(false);
  const [step, setStep] = useState<Step>("connect");
  const [connector, setConnector] = useState<Connector | null>(null);
  const [zkAccountCopied, setZkAccountCopied] = useState(false);
  const [evmAccountCopied, setEvmAccountCopied] = useState(false);

  const onBack = () => {
    if (isConnectingEvm) {
      setStep("connect");
      setConnector(null);
    }
  };

  const onSignOut = () => {
    setIsOpen(false);
    disconnectEvm();
    disconnectZk();
  };

  useEffect(() => {
    if (!addressEvm) {
      setStep("connect");
      setConnector(null);
    } else if (addressEvm && !accountZk) {
      setStep("sign");
    }
  }, [addressEvm, accountZk, isOpen]);

  return (
    <>
      <button
        onClick={() => setIsOpen(true)}
        className="flex items-center gap-2 rounded-full border h-[46px] px-4 shrink-0"
      >
        {accountZk ? (
          <>
            <img
              src="/avatar.webp"
              alt="avatar"
              className="w-[18px] h-[18px] rounded-full"
            />
            <span className="text-sm">{shortenAddress(accountZk.address)}</span>
          </>
        ) : (
          <span className="text-sm">Connect wallet</span>
        )}
      </button>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="w-[360px] rounded-3xl p-4 gap-0 pb-10">
          <div className=" mb-4 w-full flex justify-between">
            {step !== "connect" && !accountZk ? (
              <button
                onClick={onBack}
                className="ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 rounded-full bg-muted h-4 w-4 flex items-center justify-center"
              >
                <ArrowLeftIcon />
              </button>
            ) : (
              <div />
            )}

            <DialogClose className="ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 rounded-full bg-muted h-4 w-4 flex items-center justify-center">
              <XIcon />
              <span className="sr-only">Close</span>
            </DialogClose>
          </div>

          {accountZk ? (
            <>
              <DialogTitle className="text-center font-medium text-base mb-6">
                Your ZK account is ready
              </DialogTitle>

              <DialogDescription className="text-sm text-muted-foreground text-center mb-6">
                Share your ZK account address to be included in the voting. As per your EVM wallet, you'll need it to pay for gas, but feel free to change it from your provider.
              </DialogDescription>

              <div className="bg-muted rounded-md px-4 py-3 mb-2 flex items-center justify-between relative pt-6">
                <span className="text-xs text-muted-foreground absolute left-4 top-1">
                  ZK account
                </span>
                <div className="text-sm text-muted-foreground">
                  {shortenAddress(accountZk.address)}
                </div>
                <button
                  disabled={zkAccountCopied}
                  onClick={() => {
                    navigator.clipboard.writeText(accountZk.address);
                    setZkAccountCopied(true);
                    setTimeout(() => {
                      setZkAccountCopied(false);
                    }, 1000);
                  }}
                  className="text-muted-foreground absolute right-4 top-1/2 -translate-y-1/2"
                >
                  {zkAccountCopied ? (
                    <Check className="w-4 h-4" />
                  ) : (
                    <Copy className="w-4 h-4" />
                  )}
                </button>
              </div>

              <div className="bg-muted rounded-md px-4 py-3 mb-6 flex items-center justify-between relative pt-6">
                <span className="text-xs text-muted-foreground absolute left-4 top-1">
                  EVM account
                </span>
                <div className="text-sm text-muted-foreground">
                  {shortenAddress(addressEvm ?? "")}
                </div>
                <button
                  disabled={evmAccountCopied}
                  onClick={() => {
                    navigator.clipboard.writeText(addressEvm ?? "");
                    setEvmAccountCopied(true);
                    setTimeout(() => {
                      setEvmAccountCopied(false);
                    }, 1000);
                  }}
                  className="text-muted-foreground absolute right-4 top-1/2 -translate-y-1/2"
                >
                  {evmAccountCopied ? (
                    <Check className="w-4 h-4" />
                  ) : (
                    <Copy className="w-4 h-4" />
                  )}
                </button>
              </div>

              <Button onClick={onSignOut} className="w-full" size="lg">
                Sign out
              </Button>
            </>
          ) : (
            <>
              {step === "connect" && !isConnectingEvm && (
                <>
                  <DialogTitle className="text-center font-medium text-base mb-6">
                    Create or recover your ZK account
                  </DialogTitle>

                  <DialogDescription className="text-sm text-muted-foreground text-center mb-6">
                    We derive your ZK wallet from an EVM signature so you don't
                    have to keep track of one more secret.
                  </DialogDescription>

                  <div className="flex flex-col gap-2">
                    {connectorsEvm.map((connector) => (
                      <button
                        key={connector.id}
                        onClick={() => {
                          setConnector(connector);
                          connectEvm({ connector });
                        }}
                        className="border rounded-xl px-4 py-3 flex items-center gap-2 w-full"
                      >
                        <img
                          src={connector.icon}
                          alt={connector.name}
                          className="w-[18px] h-[18px]"
                        />
                        <span className="text-sm">{connector.name}</span>
                      </button>
                    ))}
                  </div>
                </>
              )}

              {step === "sign" && !isConnectingZk && (
                <>
                  <DialogTitle className="text-center font-medium text-base mb-6">
                    Sign message
                  </DialogTitle>

                  <DialogDescription className="text-sm text-muted-foreground text-center mb-6">
                    We derive your ZK wallet from an EVM signature so you don't
                    have to keep track of one more secret.
                  </DialogDescription>

                  <div className="bg-muted rounded-xl p-4 mb-6">
                    <div className="text-sm text-muted-foreground">
                      {ACCOUNT_MESSAGE}
                    </div>
                  </div>

                  <Button
                    size="lg"
                    className="w-full"
                    onClick={() => connectZk()}
                    disabled={isConnectingZk}
                  >
                    Sign and continue
                  </Button>
                </>
              )}

              {(isConnectingEvm || isConnectingZk) && (
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
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
};
