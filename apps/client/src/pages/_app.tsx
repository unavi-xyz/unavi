import "@rainbow-me/rainbowkit/styles.css";
import "../styles/globals.css";

import { httpBatchLink } from "@trpc/client/links/httpBatchLink";
import { loggerLink } from "@trpc/client/links/loggerLink";
import { withTRPC } from "@trpc/next";
import { AppType } from "next/app";
import dynamic from "next/dynamic";
import Head from "next/head";
import { Session } from "next-auth";
import React from "react";

import { AppRouter } from "../server/router";

// Export web vitals
export { reportWebVitals } from "next-axiom";

const ClientSideProviders = dynamic(() => import("../ClientSideProviders"));

const App: AppType<{ session: Session | null }> = ({
  Component,
  pageProps: { session, ...pageProps },
}) => {
  // @ts-ignore
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  return (
    <>
      <Head>
        <title>The Wired</title>

        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#ffffff" />
      </Head>

      <ClientSideProviders session={session}>
        <div className="h-screen w-full">
          {getLayout(<Component {...pageProps} />)}
        </div>
      </ClientSideProviders>
    </>
  );
};

// tRPC router
export default withTRPC<AppRouter>({
  config() {
    const url = `${getBaseUrl()}/api/trpc`;

    return {
      links: [
        loggerLink({
          enabled: (opts) =>
            process.env.NODE_ENV === "development" ||
            (opts.direction === "down" && opts.result instanceof Error),
        }),
        httpBatchLink({ url }),
      ],
      url,
      queryClientConfig: { defaultOptions: { queries: { staleTime: 60 } } },
    };
  },
})(App);

const getBaseUrl = () => {
  if (typeof window !== "undefined") return ""; // browser should use relative url
  if (process.env.VERCEL_URL) return `https://${process.env.VERCEL_URL}`; // SSR should use vercel url
  return `http://localhost:3000`; // dev SSR should use localhost
};
