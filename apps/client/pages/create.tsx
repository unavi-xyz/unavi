import Head from "next/head";
import Link from "next/link";
import { MdArrowBackIosNew } from "react-icons/md";

import Button from "../src/components/base/Button";
import { getNavbarLayout } from "../src/components/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../src/components/ui/MetaTags";

export default function Create() {
  return (
    <>
      <MetaTags title="Create" />

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex justify-center font-black text-3xl">Create</div>

          <div className="space-y-2">
            <Link href="/studio">
              <div>
                <Button variant="tonal" fullWidth squared>
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
          </div>
        </div>
      </div>
    </>
  );
}

Create.getLayout = getNavbarLayout;
