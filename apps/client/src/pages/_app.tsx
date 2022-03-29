import { IpfsProvider } from "ceramic";
import Head from "next/head";
import { QueryClientProvider } from "react-query";

import { queryClient } from "../helpers/constants";
import "../../styles/globals.css";

export default function App({ Component, pageProps }) {
  const Layout = Component.Layout ?? EmptyLayout;

  return (
    <div>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <div className="w-full h-screen">
        <QueryClientProvider client={queryClient}>
          <IpfsProvider>
            <Layout>
              <Component {...pageProps} />
            </Layout>
          </IpfsProvider>
        </QueryClientProvider>
      </div>
    </div>
  );
}

function EmptyLayout({ children }) {
  return children;
}
