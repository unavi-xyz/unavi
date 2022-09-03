import React from "react";

import Sidebar from "../src/Sidebar";
import "../styles/globals.css";

export default function App({ Component, pageProps }: any) {
  return (
    <div className="w-full h-screen overflow-hidden">
      <Sidebar>
        <Component {...pageProps} />
      </Sidebar>
    </div>
  );
}
