import { createFileRoute, useNavigate } from "@tanstack/react-router";
import boardLanding from "@/assets/board-landing.svg";
import star from "@/assets/star.svg";
import { useAccount, useConnect } from "wagmi";
import { useEffect } from "react";
import { ConnectWallet } from "@/components/connect-wallet";
import { DialogTrigger } from "@/components/ui/dialog";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const navigate = useNavigate();
  const { address } = useAccount();

  useEffect(() => {
    if (address) {
      navigate({ to: "/board" });
    }
  }, [address]);

  return (
    <div className="h-screen w-screen relative overflow-hidden">
      <h1 className="text-9xl font-black mb-4 absolute left-28 top-20">
        BATTLE
        <br />
        SHIP
      </h1>

      <ConnectWallet>
        <DialogTrigger asChild>
          <button className="h-[500px] w-[500px] absolute left-14 bottom-0 flex items-center justify-center cursor-pointer hover:scale-105 transition-transform duration-300">
            <img
              src={star}
              alt=""
              className="h-[500px] w-[500px] translate-y-1/2"
            />
            <div className="absolute left-1/2 -translate-x-1/2 bottom-16 text-3xl font-bold">
              Connect Wallet
            </div>
          </button>
        </DialogTrigger>
      </ConnectWallet>

      <img
        src={boardLanding}
        alt=""
        className="h-[170vh] absolute -right-1/2 -bottom-1/2 rotate-10"
      />
    </div>
  );
}
