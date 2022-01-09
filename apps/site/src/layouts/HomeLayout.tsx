import React from "react";
import Head from "next/head";

import HomeNavbar from "../components/HomeNavbar";

export default function HomeLayout({ children }) {
  return (
    <div>
      <Head>
        <title>The Wired - Home</title>
      </Head>

      <HomeNavbar />
      {children}
    </div>
  );
}
