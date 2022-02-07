import { useEffect } from "react";
import { CssBaseline } from "@mui/material";
import { ThemeProvider } from "@mui/material/styles";
import Head from "next/head";
import { useRouter } from "next/router";
import { CeramicProvider } from "ceramic";
import { theme } from "ui";

import "../styles/globals.css";

export default function MyApp(props) {
  const { Component, pageProps } = props;

  const Layout = Component.Layout ?? EmptyLayout;

  const router = useRouter();

  useEffect(() => {
    const prev = sessionStorage.getItem("currentPath");
    sessionStorage.setItem("prevPath", prev);
    sessionStorage.setItem("currentPath", router.asPath);
  }, [router.asPath]);

  return (
    <>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <CssBaseline />

      <ThemeProvider theme={theme}>
        <CeramicProvider>
          <Layout>
            <Component {...pageProps} />
          </Layout>
        </CeramicProvider>
      </ThemeProvider>
    </>
  );
}

function EmptyLayout({ children }) {
  return children;
}
