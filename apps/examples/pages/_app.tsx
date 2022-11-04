import "../styles/globals.css";

import Sidebar from "../src/Sidebar";

export default function App({ Component, pageProps }: any) {
  return (
    <>
      <Sidebar />
      <Component {...pageProps} />
    </>
  );
}
