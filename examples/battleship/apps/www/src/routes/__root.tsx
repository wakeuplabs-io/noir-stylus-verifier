import { createRootRoute, Outlet } from "@tanstack/react-router";
import { PrivyProvider } from "@privy-io/react-auth";

export const Route = createRootRoute({
  component: () => (
    <div>
      <div className="md:hidden h-screen w-screen flex items-center justify-center text-center">
        Auth that's awkward. Mobile is still not supported.
      </div>
      <div className="hidden md:block">
        <PrivyProvider
          appId="cmdtcys1u002kl80bqrtr24of"
          config={{
            // Create embedded wallets for users who don't have a wallet
            embeddedWallets: {
              ethereum: {
                createOnLogin: "users-without-wallets",
              },
            },
          }}
        >
          <Outlet />
        </PrivyProvider>
      </div>
    </div>
  ),
});
