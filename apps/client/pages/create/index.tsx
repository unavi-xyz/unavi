import Head from "next/head";
import { useRouter } from "next/router";

import { createNewLocalSpace } from "../../src/helpers/indexedDB/LocalSpace/helpers";
import { useLocalSpaces } from "../../src/helpers/indexedDB/LocalSpace/hooks/useLocalSpaces";

import LocalSpaceCard from "../../src/components/ui/LocalSpaceCard";
import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";

export default function Create() {
  const router = useRouter();
  const localSpaces = useLocalSpaces();

  async function handleNewSpace() {
    const { id } = await createNewLocalSpace();
    router.push(`/create/${id}`);
  }

  return (
    <div>
      <Head>
        <title>Create · The Wired</title>
      </Head>

      <div className="flex justify-center py-8 mx-8">
        <div className="max-w space-y-8">
          <div className="flex flex-col items-center justify-center">
            <div className="font-black text-3xl">Create</div>
          </div>

          <div className="w-full grid grid-cols-1 md:grid-cols-2 gap-4">
            <div
              onClick={handleNewSpace}
              className="rounded-3xl flex items-center justify-center cursor-pointer
                         aspect-card transition hover:ring-2 hover:ring-outline
                         bg-secondaryContainer text-onSecondaryContainer"
            >
              <div className="flex flex-col items-center justify-center">
                <div className="text-3xl">+</div>
                <div className="text-lg">New Space</div>
              </div>
            </div>

            {localSpaces.map((localSpace) => (
              <LocalSpaceCard key={localSpace.id} localSpace={localSpace} />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

Create.Layout = NavbarLayout;
