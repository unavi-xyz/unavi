import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";

import Button from "../../src/components/base/Button";
import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";
import { useLocalSpace } from "../../src/helpers/indexeddb/LocalSpace/hooks/useLocalScene";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const localSpace = useLocalSpace(id);

  if (!localSpace) return null;

  return (
    <>
      <Head>
        <title>{localSpace.name} · The Wired</title>
      </Head>

      <div className="mx-8 h-full">
        <div className="max-w mx-auto py-8 w-full h-full space-y-8">
          <div className="flex space-x-8">
            <div className="w-full rounded-3xl aspect-card bg-secondaryContainer">
              {localSpace?.image && (
                <img
                  src={localSpace.image}
                  alt="space preview"
                  className="object-cover rounded-3xl w-full h-full"
                />
              )}
            </div>

            <div className="w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="space-y-2">
                <div className="font-black text-3xl flex justify-center">
                  {localSpace.name}
                </div>
              </div>

              <Link href={`/studio/${localSpace.id}`} passHref>
                <div>
                  <Button variant="tonal" fullWidth>
                    <div className="py-2">Open Studio</div>
                  </Button>
                </div>
              </Link>
            </div>
          </div>
          <div>
            {localSpace?.description && (
              <div>
                <div className="text-xl font-bold">About</div>
                <div className="text-lg text-outline">
                  {localSpace.description}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
}

Id.Layout = NavbarLayout;
