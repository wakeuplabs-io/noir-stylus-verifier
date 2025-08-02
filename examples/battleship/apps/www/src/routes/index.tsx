import battleship from "@/assets/battleship-white.svg";
import { StarsBackground } from "@/components/animate-ui/backgrounds/stars";
import { cn } from "@/lib/utils";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { usePrivy } from "@privy-io/react-auth";
import { useEffect } from "react";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const navigate = useNavigate();
  const { ready, authenticated, user, login } = usePrivy();

  useEffect(() => {
    if (authenticated) {
      navigate({ to: "/board" });
    }
  }, [authenticated]);

  return (
    <>
      <div className="p-2 h-screen w-screen flex justify-center items-center relative z-10">
        <div className="relative">
          <div className="mb-6 uppercase text-[#FAEC00] text-2xl text-center">
            The classic naval combat game
          </div>
          <div className="mb-6">
            <img
              src={battleship}
              alt="Battleship"
              className="w-[640px] h-[110px]"
            />
          </div>
          <div className="mb-12 uppercase text-[#0CABE8] text-2xl text-center font-bold">
            Noir x Stylus edition
          </div>

          {!ready ? (
            <div className="">
              <div className="border border-white rounded-lg p-2 w-[675px]">
                <div className="h-[50px] bg-[#FF0055] rounded-lg w-full flex justify-center items-center text-white font-bold  text-xl">
                  <span>LOADING...</span>
                </div>
              </div>
            </div>
          ) : (
            <div className="border border-white rounded-lg p-2 w-[675px]">
              <button
                onClick={() => login()}
                className="h-[50px] bg-[#FF0055] rounded-lg w-full flex justify-center items-center text-white font-bold  text-xl"
              >
                <span>LOGIN</span>
              </button>
            </div>
          )}

          <Battleship
            size={3}
            ship="cruiser"
            className="absolute -top-40 -left-20"
          />
          <Battleship
            size={2}
            ship="destroyer"
            className="absolute -top-10 -right-56 rotate-90"
          />
          <Battleship
            size={3}
            ship="cruiser"
            className="absolute -bottom-20 -right-20"
          />
          <Battleship
            size={3}
            ship="cruiser"
            className="absolute -bottom-20 -right-20"
          />
          <Battleship
            size={5}
            ship="cruiser"
            className="absolute bottom-32 -left-56 rotate-90"
          />
        </div>

        <Battleship
          size={2}
          ship="destroyer"
          className="absolute top-0 left-20 rotate-90"
        />
      </div>

      <StarsBackground className="absolute inset-0 flex items-center justify-center rounded-xl" />
    </>
  );
}

export const Battleship: React.FC<{
  size: number;
  ship: "cruiser" | "destroyer";
  className?: string;
}> = ({ size, ship, className }) => {
  return (
    <div className="flex items-center">
      <div
        className={cn(
          `bg-battleship flex rounded-lg h-[32px] items-center justify-around`,
          ship === "cruiser" && "rounded-l-full",
          className
        )}
        style={{ width: `${size * 32}px` }}
      >
        {Array.from({ length: size }).map((_, i) => (
          <div key={i} className="bg-hit h-[12px] w-[12px] rounded-full" />
        ))}
      </div>
    </div>
  );
};
