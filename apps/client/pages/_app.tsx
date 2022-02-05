import * as React from "react";
import Head from "next/head";

import "../styles/globals.css";
import { CeramicProvider } from "ceramic";
import { MultiplayerProvider } from "3d";

export default function MyApp(props) {
  const { Component, pageProps } = props;

  return (
    <>
      <Head>
        <title>The Wired - App</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <CeramicProvider>
        <MultiplayerProvider>
          <Component {...pageProps} />
        </MultiplayerProvider>
      </CeramicProvider>
    </>
  );
}
