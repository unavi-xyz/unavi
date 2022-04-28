import Head from "next/head";
import NavbarLayout from "../src/components/layouts/NavbarLayout/NavbarLayout";

export default function Explore() {
  return (
    <div>
      <Head>
        <title>Explore · The Wired</title>
      </Head>
    </div>
  );
}

Explore.Layout = NavbarLayout;
