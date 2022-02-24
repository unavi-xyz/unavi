import Head from "next/head";
import { CeramicProvider } from "ceramic";

import "../styles/globals.css";
import Sidebar from "../src/components/Sidebar/Sidebar";

export default function App({ Component, pageProps }) {
  return (
    <div>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <link rel="shortcut icon" href="/favicon.ico" />
      </Head>

      <CeramicProvider>
        <Sidebar />
        <div className="ml-16 p-12">
          <Component {...pageProps} />
        </div>
      </CeramicProvider>
    </div>
  );
}
