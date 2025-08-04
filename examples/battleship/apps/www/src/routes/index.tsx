import { createFileRoute, useNavigate } from "@tanstack/react-router";
import boardLanding from "@/assets/board-landing.svg";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const navigate = useNavigate();

  return (
    <div className="h-screen w-screen relative">
      <div className="p-10">
        <h1 className="text-7xl font-black mb-4">BATTLESHIP</h1>
        <p className="text-2xl font-bold mb-6">
          A multiplayer battleship game built <br /> with Noir, Stylus and Noir
          Stylus Verifier
        </p>

        <button className="relative px-6 py-3 text-xl font-bold text-black bg-[#FF8577] border-3 border-black rounded-full shadow-[4px_4px_0px_#000] active:shadow-none active:translate-x-[6px] active:translate-y-[6px] transition-transform duration-100">
          Login <span className="ml-2">→</span>
        </button>
      </div>
      <img
        src={boardLanding}
        alt=""
        className="h-[170vh] absolute -right-1/2 -bottom-1/2 rotate-10"
      />
    </div>
  );
}
