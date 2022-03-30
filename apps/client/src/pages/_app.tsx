import { IpfsProvider } from "ceramic";
import Head from "next/head";
import { QueryClientProvider } from "react-query";

import "../../styles/globals.css";
import SocketProvider from "../components/app/SocketProvider";
import { queryClient } from "../helpers/constants";

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
            <SocketProvider>
              <Layout>
                <Component {...pageProps} />
              </Layout>
            </SocketProvider>
          </IpfsProvider>
        </QueryClientProvider>
      </div>
    </div>
  );
}

function EmptyLayout({ children }) {
  return children;
}
