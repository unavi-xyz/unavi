import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";

import Button from "../../src/components/base/Button";
import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";
import { useMediaImage } from "../../src/helpers/lens/hooks/useMediaImage";
import { usePublication } from "../../src/helpers/lens/hooks/usePublication";

export default function Space() {
  const router = useRouter();
  const id = router.query.id as string;

  const publication = usePublication(id);
  const image = useMediaImage(publication?.metadata.media[0]);

  return (
    <>
      <Head>
        <title>{publication?.metadata.name ?? id} · The Wired</title>
      </Head>

      <div className="mx-8 h-full">
        <div className="max-w mx-auto py-8 w-full h-full space-y-8">
          <div className="flex space-x-8">
            <div className="w-full rounded-3xl aspect-card bg-secondaryContainer">
              {image && (
                <img
                  src={image}
                  alt="space preview"
                  className="object-cover rounded-3xl w-full h-full"
                />
              )}
            </div>

            <div className="w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="space-y-4">
                <div className="font-black text-3xl flex justify-center">
                  {publication?.metadata.name}
                </div>

                <div className="space-y-2">
                  <div className="font-bold flex space-x-1">
                    <div>By</div>
                    <Link href={`/user/${publication?.profile.handle}`}>
                      <div className="hover:underline cursor-pointer">
                        @{publication?.profile.handle}
                      </div>
                    </Link>
                  </div>
                </div>
              </div>

              <Link href={`/app/${id}`} passHref>
                <div>
                  <Button variant="tonal" fullWidth>
                    <div className="py-2">Join Space</div>
                  </Button>
                </div>
              </Link>
            </div>
          </div>

          <div>
            {publication?.metadata.description.length > 0 && (
              <div>
                <div className="text-xl font-bold">About</div>
                <div className="text-lg text-outline">
                  {publication?.metadata.description}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
}

Space.Layout = NavbarLayout;
