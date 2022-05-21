import Head from "next/head";
import React from "react";
import { Provider } from "urql";

import { lensClient } from "../src/helpers/lens/client";
import "../styles/globals.css";

export default function App({ Component, pageProps }: any) {
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  return (
    <div>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#6750A4"></meta>
      </Head>

      <div className="w-full h-screen">
        <Provider value={lensClient}>
          {getLayout(<Component {...pageProps} />)}
        </Provider>
      </div>
    </div>
  );
}
