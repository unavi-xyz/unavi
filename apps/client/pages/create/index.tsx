import Head from "next/head";
import Link from "next/link";
import { MdArrowBackIosNew } from "react-icons/md";

import Button from "../../src/components/base/Button";
import { getNavbarLayout } from "../../src/components/layouts/NavbarLayout/NavbarLayout";

export default function Create() {
  return (
    <div>
      <Head>
        <title>Create / The Wired</title>
      </Head>

      <div className="flex justify-center py-8 mx-8">
        <div className="max-w space-y-8">
          <div className="flex justify-center font-black text-3xl">Create</div>

          <div className="space-y-2">
            <Link href="/studio">
              <div>
                <Button variant="text" fullWidth squared>
                  <div className="p-2 flex items-center justify-between text-lg">
                    <div>
                      <div className="flex">Studio</div>
                      <div className="flex font-normal">
                        A visual editor for creating spaces
                      </div>
                    </div>

                    <MdArrowBackIosNew className="rotate-180" />
                  </div>
                </Button>
              </div>
            </Link>

            <Button variant="text" fullWidth squared>
              <div className="p-2 flex items-center justify-between text-lg">
                <div>
                  <div className="flex">Publish</div>
                  <div className="flex font-normal">Mint a new space NFT</div>
                </div>

                <MdArrowBackIosNew className="rotate-180" />
              </div>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

Create.getLayout = getNavbarLayout;
