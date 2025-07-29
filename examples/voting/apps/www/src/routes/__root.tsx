import { Footer } from "@/components/footer";
import { createRootRoute, Outlet } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: () => (
    <div>
      <div className="max-w-5xl mx-auto border-x min-h-screen w-full flex flex-col">
        <div className="flex-1 h-full">
          <Outlet />
        </div>

        <Footer />
      </div>
    </div>
  ),
});
