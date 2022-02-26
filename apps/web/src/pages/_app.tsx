import Head from "next/head";

import "../../styles/globals.css";
import Sidebar from "../components/Sidebar/Sidebar";

export default function App({ Component, pageProps }) {
  const Layout = Component.Layout ?? EmptyLayout;

  return (
    <div>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <div className="flex">
        <Sidebar />

        <div className="w-full bg-neutral-100 h-screen overflow-hidden">
          <Layout>
            <Component {...pageProps} />
          </Layout>
        </div>
      </div>
    </div>
  );
}

function EmptyLayout({ children }) {
  return children;
}
