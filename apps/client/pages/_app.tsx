import Head from "next/head";
import { useRouter } from "next/router";
import React, { useEffect } from "react";
import { Provider } from "urql";

import * as gtag from "../src/helpers/gtag";
import { useAutoLogin } from "../src/helpers/ethers/useAutoLogin";
import { lensClient } from "../src/helpers/lens/client";
import "../styles/globals.css";

export default function App({ Component, pageProps }: any) {
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  const router = useRouter();

  useEffect(() => {
    function handleRouteChange(url: URL) {
      gtag.pageview(url);
    }

    router.events.on("routeChangeComplete", handleRouteChange);
    router.events.on("hashChangeComplete", handleRouteChange);
    return () => {
      router.events.off("routeChangeComplete", handleRouteChange);
      router.events.off("hashChangeComplete", handleRouteChange);
    };
  }, [router.events]);

  useAutoLogin();

  return (
    <div>
      <Head>
        <title>The Wired</title>

        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#fffbfe" />
      </Head>

      <div className="w-full h-screen">
        <Provider value={lensClient}>
          {getLayout(<Component {...pageProps} />)}
        </Provider>
      </div>
    </div>
  );
}
