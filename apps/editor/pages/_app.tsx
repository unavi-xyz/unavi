import * as React from "react";
import { CssBaseline } from "@mui/material";
import { ThemeProvider } from "@mui/material/styles";
import Head from "next/head";
import { ClientProvider } from "matrix";

import theme from "../src/ui/theme";

import "../styles/globals.css";

export default function MyApp(props) {
  const { Component, pageProps } = props;

  const Layout = Component.Layout ?? EmptyLayout;

  return (
    <>
      <Head>
        <title>The Wired - Editor</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <CssBaseline />

      <ThemeProvider theme={theme}>
        <ClientProvider>
          <Layout>
            <Component {...pageProps} />
          </Layout>
        </ClientProvider>
      </ThemeProvider>
    </>
  );
}

function EmptyLayout({ children }) {
  return children;
}
