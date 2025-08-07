import { createRootRoute, Outlet } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: () => (
    <div>
      <div className="md:hidden h-screen w-screen flex items-center justify-center text-center">
        Auth that's awkward. Mobile is still not supported.
      </div>
      <div className="hidden md:block">
        <Outlet />
      </div>
    </div>
  ),
});
