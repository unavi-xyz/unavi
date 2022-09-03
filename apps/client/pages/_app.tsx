import {
  RainbowKitProvider,
  connectorsForWallets,
  wallet,
} from "@rainbow-me/rainbowkit";
import "@rainbow-me/rainbowkit/styles.css";
import { withTRPC } from "@trpc/next";
import { SessionProvider } from "next-auth/react";
import Head from "next/head";
import React from "react";
import { WagmiConfig, chain, configureChains, createClient } from "wagmi";
import { alchemyProvider } from "wagmi/providers/alchemy";
import { publicProvider } from "wagmi/providers/public";

import { IpfsProvider } from "@wired-labs/ipfs";
import { LensProvider } from "@wired-labs/lens";

import LoginProvider from "../src/login/LoginProvider";
import { theme } from "../src/login/theme";
import "../styles/globals.css";
import { AppRouter } from "./api/trpc/[trpc]";

// Export web vitals
export { reportWebVitals } from "next-axiom";

// RainbowKit / Wagmi
const apiKey = process.env.ALCHEMY_ID;

const { chains, provider } = configureChains(
  [chain.polygonMumbai],
  [alchemyProvider({ apiKey }), publicProvider()]
);

const needsInjectedWalletFallback =
  typeof window !== "undefined" &&
  window.ethereum &&
  !window.ethereum.isMetaMask &&
  !window.ethereum.isCoinbaseWallet;

const connectors = connectorsForWallets([
  {
    groupName: "Popular",
    wallets: [
      wallet.metaMask({ chains }),
      wallet.rainbow({ chains }),
      wallet.coinbase({ chains, appName: "The Wired" }),
    ],
  },
  {
    groupName: "More",
    wallets: [
      wallet.argent({ chains }),
      wallet.ledger({ chains }),
      wallet.walletConnect({ chains }),
      ...(needsInjectedWalletFallback ? [wallet.injected({ chains })] : []),
    ],
  },
]);

export const wagmiClient: any = createClient({
  connectors,
  provider,
});

// App
function App({ Component, pageProps: { session, ...pageProps } }: any) {
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  return (
    <>
      <Head>
        <title>The Wired</title>

        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#ffffff" />
      </Head>

      <SessionProvider session={session}>
        <WagmiConfig client={wagmiClient}>
          <RainbowKitProvider theme={theme} chains={chains}>
            <IpfsProvider>
              <LensProvider>
                <LoginProvider>
                  <div className="w-full h-screen">
                    {getLayout(<Component {...pageProps} />)}
                  </div>
                </LoginProvider>
              </LensProvider>
            </IpfsProvider>
          </RainbowKitProvider>
        </WagmiConfig>
      </SessionProvider>
    </>
  );
}

// tRPC router
export default withTRPC<AppRouter>({
  config({ ctx }) {
    const url = `${getBaseUrl()}/api/trpc`;

    return {
      url,
      queryClientConfig: { defaultOptions: { queries: { staleTime: 60 } } },
    };

    ssr: true;
  },
})(App);

const getBaseUrl = () => {
  if (typeof window !== "undefined") return ""; // browser should use relative url
  if (process.env.VERCEL_URL) return `https://${process.env.VERCEL_URL}`; // SSR should use vercel url
  return `http://localhost:3000`; // dev SSR should use localhost
};
