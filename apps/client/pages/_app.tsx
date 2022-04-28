import { IpfsProvider } from "ceramic";
import Head from "next/head";
import { QueryClientProvider } from "react-query";

import { queryClient } from "../src/helpers/constants";

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

      <div className="w-full h-screen">
        <QueryClientProvider client={queryClient}>
          <IpfsProvider>
            <SecondLayout>
              <Layout>
                <Component {...pageProps} />
              </Layout>
            </SecondLayout>
          </IpfsProvider>
        </QueryClientProvider>
      </div>
    </div>
  );
}

function EmptyLayout({ children }: { children: React.ReactNode }) {
  return children;
}
