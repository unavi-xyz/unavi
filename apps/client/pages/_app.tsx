import Head from "next/head";
import React from "react";

import { EthersProvider } from "@wired-xr/ethers";
import { IpfsProvider } from "@wired-xr/ipfs";
import { LensProvider, useAutoLogin } from "@wired-xr/lens";

import "../styles/globals.css";

export default function App({ Component, pageProps }: any) {
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
              <AutoLogin>{getLayout(<Component {...pageProps} />)}</AutoLogin>
            </LensProvider>
          </EthersProvider>
        </IpfsProvider>
      </div>
    </>
  );
}

function AutoLogin({ children }: { children: React.ReactNode }) {
  useAutoLogin();

  return <>{children}</>;
}
