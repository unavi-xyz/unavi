import * as React from "react";
import Head from "next/head";
import { ThemeProvider } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";

import "../styles/globals.css";
import theme from "../src/theme";
import { MatrixProvider } from "matrix";

export default function MyApp(props) {
  const { Component, pageProps } = props;

  const Layout = Component.Layout ?? EmptyLayout;

  return (
    <>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>
      <ThemeProvider theme={theme}>
        <MatrixProvider>
          <>
            <CssBaseline />
            <Layout>
              <Component {...pageProps} />
            </Layout>
          </>
        </MatrixProvider>
      </ThemeProvider>
    </>
  );
}

function EmptyLayout({ children }) {
  return children;
}
