import Head from "next/head";

import "../../styles/globals.css";
import Sidebar from "../components/Sidebar/Sidebar";

export default function App({ Component, pageProps }) {
  return (
    <div>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <Sidebar />
      <div className="ml-16 bg-neutral-100 h-screen">
        <Component {...pageProps} />
      </div>
    </div>
  );
}
