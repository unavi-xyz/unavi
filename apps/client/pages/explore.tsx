import Head from "next/head";
import NavbarLayout from "../src/components/layouts/NavbarLayout/NavbarLayout";

export default function Explore() {
  return (
    <div>
      <Head>
        <title>Explore · The Wired</title>
      </Head>

      <div className="flex justify-center py-8 mx-8">
        <div className="max-w space-y-8">
          <div className="flex flex-col items-center justify-center">
            <div className="font-black text-3xl">Explore</div>
          </div>
        </div>
      </div>
    </div>
  );
}

Explore.Layout = NavbarLayout;
