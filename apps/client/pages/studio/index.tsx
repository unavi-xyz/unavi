import Head from "next/head";
import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";

export default function Studio() {
  return (
    <div>
      <Head>
        <title>Studio · The Wired</title>
      </Head>
    </div>
  );
}

Studio.Layout = NavbarLayout;
