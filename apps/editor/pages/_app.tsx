import * as React from "react";
import { CssBaseline } from "@mui/material";
import { ThemeProvider } from "@mui/material/styles";
import Head from "next/head";

import { ClientProvider } from "matrix";
import theme from "../src/theme";

import "../styles/globals.css";

export default function MyApp(props) {
  const { Component, pageProps } = props;

  return (
    <>
      <Head>
        <title>The Wired - Editor</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <CssBaseline />

      <ThemeProvider theme={theme}>
        <ClientProvider>
          <Component {...pageProps} />
        </ClientProvider>
      </ThemeProvider>
    </>
  );
}
