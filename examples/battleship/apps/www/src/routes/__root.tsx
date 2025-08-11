import { createRootRoute, Outlet } from "@tanstack/react-router";
import { Toaster } from "@/components/ui/sonner";

export const Route = createRootRoute({
  component: () => (
    <div>
      <div className="lg:hidden h-screen w-screen flex items-center justify-center text-center">
        This app is not supported in small screens. Try from a computer....
      </div>
      <div className="hidden lg:block">
        <Outlet />
        <Toaster />
      </div>
    </div>
  ),
});
