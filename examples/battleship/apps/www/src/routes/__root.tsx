import { createRootRoute, Outlet } from "@tanstack/react-router";
import { Toaster } from "@/components/ui/sonner";

export const Route = createRootRoute({
  component: () => (
    <div>
      <div className="lg:hidden h-screen w-screen flex items-center justify-center text-center">
        Auth that's awkward. Mobile is still not supported.
      </div>
      <div className="hidden lg:block">
        <Outlet />
        <Toaster />
      </div>
    </div>
  ),
});
