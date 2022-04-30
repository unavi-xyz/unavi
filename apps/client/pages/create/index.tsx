import Head from "next/head";
import { useRouter } from "next/router";
import { customAlphabet } from "nanoid";

import { LocalSpace } from "../../src/helpers/indexeddb/localSpaces/types";
import { createLocalSpace } from "../../src/helpers/indexeddb/localSpaces/db";
import { useLocalSpaces } from "../../src/helpers/indexeddb/localSpaces/useLocalSpaces";

import LocalSpaceCard from "../../src/components/ui/LocalSpaceCard";
import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";

const nanoid = customAlphabet("0123456789", 12);

export default function Create() {
  const router = useRouter();
  const localSpaces = useLocalSpaces();

  async function handleNewSpace() {
    const localSpace: LocalSpace = {
      id: nanoid(),
      name: "My Space",
      description: "",
      scene: {} as any,
    };

    await createLocalSpace(localSpace);

    router.push(`/create/${localSpace.id}`);
  }

  return (
    <div>
      <Head>
        <title>Create · The Wired</title>
      </Head>

      <div className="flex justify-center py-8">
        <div className="max-w space-y-8">
          <div className="flex flex-col items-center justify-center space-x-8">
            <div className="font-black text-3xl">Create</div>
          </div>

          <div className="bg-white w-full rounded-3xl border grid grid-cols-4 gap-4 p-8">
            {localSpaces.map((localSpace) => (
              <LocalSpaceCard key={localSpace.id} localSpace={localSpace} />
            ))}

            <div
              onClick={handleNewSpace}
              className="h-96 rounded-3xl border border-dashed flex items-center justify-center
                         hover:shadow-lg transition-all cursor-pointer duration-500
                         border-neutral-300"
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
