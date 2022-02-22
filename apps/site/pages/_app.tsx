import { useEffect } from "react";
import { CssBaseline } from "@mui/material";
import { ThemeProvider } from "@mui/material/styles";
import Head from "next/head";
import { useRouter } from "next/router";
import { CeramicProvider } from "ceramic";

import "../styles/globals.css";
import { theme } from "../src/theme";

export default function MyApp(props) {
  const { Component, pageProps } = props;

  const Layout = Component.Layout ?? EmptyLayout;

  const router = useRouter();

  useEffect(() => {
    sessionStorage.setItem("pathHistory", "[]");
  }, []);

  useEffect(() => {
    if (router.asPath !== window.location.pathname) return;
    const history = JSON.parse(sessionStorage.getItem("pathHistory")) ?? [];
    if (history[history.length - 1] === window.location.pathname) return;
    const newHistory = JSON.stringify([...history, window.location.pathname]);
    sessionStorage.setItem("pathHistory", newHistory);
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
