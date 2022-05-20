import Head from "next/head";
import { Provider } from "urql";

import { lensClient } from "../src/helpers/lens/client";
import "../styles/globals.css";

export default function App({ Component, pageProps }: any) {
  const Layout = Component.Layout ?? EmptyLayout;
  const SecondLayout = Layout.Layout ?? EmptyLayout;

  return (
    <div>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <div
        className="w-full h-screen"
        style={{ paddingLeft: "calc(100vw - 100%)" }}
      >
        <Provider value={lensClient}>
          <SecondLayout>
            <Layout>
              <Component {...pageProps} />
            </Layout>
          </SecondLayout>
        </Provider>
      </div>
    </div>
  );
}

function EmptyLayout({ children }: { children: React.ReactNode }) {
  return children;
}
