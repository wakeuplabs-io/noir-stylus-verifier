import "./index.css";
import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { routeTree } from "./routeTree.gen";
import { http, createConfig, WagmiProvider } from "wagmi";
import { arbitrumSepolia } from "wagmi/chains";
import { metaMask } from "wagmi/connectors";
import { ZkAccountProvider } from "@/hooks/account";

const router = createRouter({ routeTree });
const queryClient = new QueryClient();

const wagmiConfig = createConfig({
  chains: [arbitrumSepolia],
  transports: {
    [arbitrumSepolia.id]: http(),
  },
  connectors: [metaMask()],
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const rootElement = document.getElementById("root")!;
if (!rootElement.innerHTML) {
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <StrictMode>
      <WagmiProvider config={wagmiConfig}>
        <QueryClientProvider client={queryClient}>
          <ZkAccountProvider>
            <RouterProvider router={router} />
          </ZkAccountProvider>
        </QueryClientProvider>
      </WagmiProvider>
    </StrictMode>
  );
}
