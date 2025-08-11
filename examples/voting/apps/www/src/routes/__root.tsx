import { Footer } from "@/components/footer";
import { createRootRoute, Outlet } from "@tanstack/react-router";
import { Toaster } from "@/components/ui/sonner";

export const Route = createRootRoute({
  component: () => (
    <div>
      <div className="lg:hidden h-screen w-screen flex items-center justify-center text-center">
        This app is not supported on small screens. Try from a computer....
      </div>
      <div className="hidden lg:block">
        <div>
          <div className="max-w-5xl mx-auto border-x min-h-screen w-full flex flex-col">
            <div className="flex-1 h-full">
              <Outlet />
            </div>

            <Footer />
          </div>

          <Toaster />
        </div>
      </div>
    </div>
  ),
});
