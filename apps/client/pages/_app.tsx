import { withTRPC } from "@trpc/next";
import Head from "next/head";
import React from "react";

import { EthersProvider } from "@wired-xr/ethers";
import { IpfsProvider } from "@wired-xr/ipfs";
import { LensProvider } from "@wired-xr/lens";

import LoginProvider from "../src/trpc/LoginProvider";
import { useJWTStore } from "../src/trpc/store";
import "../styles/globals.css";
import { AppRouter } from "./api/trpc/[trpc]";

function App({ Component, pageProps }: any) {
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  return (
    <>
      <Head>
        <title>The Wired</title>

        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#ffffff" />
      </Head>

      <div className="w-full h-screen">
        <IpfsProvider>
          <EthersProvider>
            <LensProvider>
              <LoginProvider>{getLayout(<Component {...pageProps} />)}</LoginProvider>
            </LensProvider>
          </EthersProvider>
        </IpfsProvider>
      </div>
    </>
  );
}

export default withTRPC<AppRouter>({
  config({ ctx }) {
    const url = process.env.VERCEL_URL
      ? `https://${process.env.VERCEL_URL}/api/trpc`
      : "http://localhost:3000/api/trpc";

    return {
      url,
      queryClientConfig: { defaultOptions: { queries: { staleTime: 60 } } },
      headers() {
        const token = useJWTStore.getState().token;

        return {
          Authorization: `Bearer ${token}`,
        };
      },
    };
  },
  ssr: true,
})(App);
