import "../styles/globals.css";

import Sidebar from "../src/Sidebar";

export default function App({ Component, pageProps }: any) {
  return (
    <div className="h-screen w-full overflow-hidden">
      <Sidebar>
        <Component {...pageProps} />
      </Sidebar>
    </div>
  );
}
