import "@rainbow-me/rainbowkit/styles.css";
import { withTRPC } from "@trpc/next";
import dynamic from "next/dynamic";
import Head from "next/head";
import React from "react";

import "../styles/globals.css";
import { AppRouter } from "./api/trpc/[trpc]";

// Export web vitals
export { reportWebVitals } from "next-axiom";

const ClientSideProviders = dynamic(() => import("../src/ClientSideProviders"));

function App({ Component, pageProps: { session, ...pageProps } }: any) {
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  return (
    <>
      <Head>
        <title>The Wired</title>

        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#ffffff" />
      </Head>

      <ClientSideProviders session={session}>
        <div className="w-full h-screen">
          {getLayout(<Component {...pageProps} />)}
        </div>
      </ClientSideProviders>
    </>
  );
}

// tRPC router
export default withTRPC<AppRouter>({
  config() {
    const url = `${getBaseUrl()}/api/trpc`;

    return {
      url,
      queryClientConfig: { defaultOptions: { queries: { staleTime: 60 } } },
    };
  },
  ssr: true,
})(App);

const getBaseUrl = () => {
  if (typeof window !== "undefined") return ""; // browser should use relative url
  if (process.env.VERCEL_URL) return `https://${process.env.VERCEL_URL}`; // SSR should use vercel url
  return `http://localhost:3000`; // dev SSR should use localhost
};
