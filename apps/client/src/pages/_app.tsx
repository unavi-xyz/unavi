import "@rainbow-me/rainbowkit/styles.css";
import "../styles/globals.css";

import { Nunito } from "@next/font/google";
import { httpLink } from "@trpc/client";
import { httpBatchLink } from "@trpc/client/links/httpBatchLink";
import { loggerLink } from "@trpc/client/links/loggerLink";
import { splitLink } from "@trpc/client/links/splitLink";
import { withTRPC } from "@trpc/next";
import { AppType } from "next/app";
import dynamic from "next/dynamic";
import Head from "next/head";
import { Session } from "next-auth";
import React from "react";

import { CACHED_PATHS } from "../constants";
import { AppRouter } from "../server/router/_app";
import { getBaseUrl } from "../utils/getBaseUrl";

// Export web vitals
export { reportWebVitals } from "next-axiom";

const Toaster = dynamic(() => import("react-hot-toast").then((mod) => mod.Toaster));
const ClientSideProviders = dynamic(() => import("../client/ClientSideProviders"));

const font = Nunito({
  subsets: ["latin"],
});

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

      <div className={font.className}>
        <ClientSideProviders session={session}>
          {getLayout(<Component {...pageProps} />)}
        </ClientSideProviders>

        <Toaster position="bottom-center" />
      </div>
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

        splitLink({
          condition(op) {
            return CACHED_PATHS.includes(op.path);
          },
          true: httpLink({ url }),
          false: httpBatchLink({ url }),
        }),
      ],
      url,
      queryClientConfig: { defaultOptions: { queries: { staleTime: 60 } } },
    };
  },
})(App);
