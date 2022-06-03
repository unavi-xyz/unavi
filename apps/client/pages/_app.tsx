import Head from "next/head";
import React from "react";
import { Provider } from "urql";

import { useAutoLogin } from "../src/helpers/ethers/useAutoLogin";
import { lensClient } from "../src/helpers/lens/client";
import "../styles/globals.css";

export default function App({ Component, pageProps }: any) {
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  useAutoLogin();

  return (
    <div>
      <Head>
        <title>The Wired</title>
      </Head>

      <div className="w-full h-screen">
        <Provider value={lensClient}>
          {getLayout(<Component {...pageProps} />)}
        </Provider>
      </div>
    </div>
  );
}
