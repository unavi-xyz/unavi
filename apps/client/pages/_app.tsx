import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import "@rainbow-me/rainbowkit/styles.css";
import "@rainbow-me/rainbowkit/styles.css";
import { withTRPC } from "@trpc/next";
import { SessionProvider } from "next-auth/react";
import Head from "next/head";
import React from "react";
import { WagmiConfig } from "wagmi";

import { IpfsProvider } from "@wired-labs/ipfs";
import { LensProvider } from "@wired-labs/lens";

import LoginProvider from "../src/login/LoginProvider";
import { RainbowAuthProvider } from "../src/login/RainbowAuthProvider";
import { theme } from "../src/login/theme";
import { chains, wagmiClient } from "../src/login/wagmi";
import "../styles/globals.css";
import { AppRouter } from "./api/trpc/[trpc]";

// Export web vitals
export { reportWebVitals } from "next-axiom";

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

      <WagmiConfig client={wagmiClient}>
        <SessionProvider session={session}>
          <RainbowAuthProvider>
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
          </RainbowAuthProvider>
        </SessionProvider>
      </WagmiConfig>
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
