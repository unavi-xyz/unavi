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
          <div className="rounded-3xl h-72 ">
            {image && (
              <img
                src={image}
                alt="space preview"
                className="object-cover rounded-3xl w-full h-full"
              />
            )}
          </div>

          <div className="w-full min-w-fit flex flex-col space-y-8">
            <div className="font-black text-3xl flex justify-center">
              {publication?.metadata.name}
            </div>
            {publication?.metadata.description.length > 0 && (
              <div>{publication?.metadata.description}</div>
            )}
            <Link href={`/app/${id}`} passHref>
              <div>
                <Button variant="outlined" fullWidth>
                  Join Space
                </Button>
              </div>
            </Link>
          </div>
        </div>
      </div>
    </>
  );
}

Space.Layout = NavbarLayout;
