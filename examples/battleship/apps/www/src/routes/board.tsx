import { createFileRoute, Navigate } from "@tanstack/react-router";
import battleship from "@/assets/battleship-black.svg";
import { usePrivy } from "@privy-io/react-auth";

export const Route = createFileRoute("/board")({
  component: RouteComponent,
});

function RouteComponent() {
  const { logout, authenticated } = usePrivy();

  if (!authenticated) {
    return <Navigate to="/" />;
  }

  return (
    <div>
      <header className="border-b h-12 flex items-center px-4 w-full bg-muted">
        <img src={battleship} alt="logo" />

        <button onClick={() => logout()}>Sign out</button>
      </header>

      <div className="grid gird-cols-2">

        <div>
          <div>Your fleet</div>
        </div>

      </div>
    </div>
  );
}
