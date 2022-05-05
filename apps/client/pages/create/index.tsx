import Head from "next/head";
import { useRouter } from "next/router";

import { createNewLocalSpace } from "../../src/helpers/indexedDB/localSpaces/helpers";
import { useLocalSpaces } from "../../src/helpers/indexeddb/localSpaces/hooks/useLocalSpaces";

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
          <div className="flex flex-col items-center justify-center space-x-8">
            <div className="font-black text-3xl">Create</div>
          </div>

          <div className="w-full grid grid-cols-3 gap-4">
            {localSpaces.map((localSpace) => (
              <LocalSpaceCard key={localSpace.id} localSpace={localSpace} />
            ))}

            <div
              onClick={handleNewSpace}
              className="rounded-3xl border border-dashed flex items-center justify-center
                         hover:shadow-lg transition-all cursor-pointer duration-300
                         border-neutral-300 aspect-card"
            >
              <div className="text-neutral-500 flex flex-col items-center justify-center">
                <div className="text-3xl">+</div>
                <div className="text-lg">New Space</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

Create.Layout = NavbarLayout;
