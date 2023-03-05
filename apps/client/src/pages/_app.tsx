import "@rainbow-me/rainbowkit/styles.css";
import "../../styles/globals.css";

import { AppType } from "next/app";
import dynamic from "next/dynamic";
import { Nunito } from "next/font/google";
import Head from "next/head";
import { Session } from "next-auth";
import React from "react";

// Export web vitals
export { reportWebVitals } from "next-axiom";

const Toaster = dynamic(() => import("react-hot-toast").then((mod) => mod.Toaster));
const ClientSideProviders = dynamic(() => import("../client/ClientSideProviders"));

const font = Nunito({ subsets: ["latin"] });

const App: AppType<{ session: Session | null }> = ({
  Component,
  pageProps: { session, ...pageProps },
}) => {
  // @ts-ignore
  const getLayout = Component.getLayout || ((page: React.ReactNode) => page);

  return (
    <>
      <Head>
        <title>The Wired</title>

        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="theme-color" content="#ffffff" />
      </Head>

      <div className={font.className}>
        <ClientSideProviders session={session}>
          {getLayout(<Component {...pageProps} />)}
        </ClientSideProviders>

        <Toaster position="bottom-center" />
      </div>
    </>
  );
};

export default App;
