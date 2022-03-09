import Head from "next/head";
import { QueryClient, QueryClientProvider } from "react-query";

import "../../styles/globals.css";

const queryClient = new QueryClient();

export default function App({ Component, pageProps }) {
  const Layout = Component.Layout ?? EmptyLayout;

  return (
    <div>
      <Head>
        <title>The Wired</title>
        <meta name="viewport" content="initial-scale=1, width=device-width" />
      </Head>

      <div className="w-full bg-neutral-100 h-screen">
        <QueryClientProvider client={queryClient}>
          <Layout>
            <Component {...pageProps} />
          </Layout>
        </QueryClientProvider>
      </div>
    </div>
  );
}

function EmptyLayout({ children }) {
  return children;
}
