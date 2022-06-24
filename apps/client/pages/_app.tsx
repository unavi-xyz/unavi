import Head from "next/head";
import React from "react";
import { Provider } from "urql";

import { useAutoLogin } from "../src/lib/ethers/useAutoLogin";
import { lensClient } from "../src/lib/lens/client";
import "../styles/globals.css";

export default function App({ Component, pageProps }: any) {
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  useAutoLogin();

  return (
    <>
      <Head>
        <title>The Wired</title>

        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#ffffff" />
      </Head>

      <div className="w-full h-screen">
        <Provider value={lensClient}>
          {getLayout(<Component {...pageProps} />)}
        </Provider>
      </div>
    </>
  );
}
