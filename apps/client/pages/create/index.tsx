import Head from "next/head";
import { useRouter } from "next/router";
import { MdOutlineAdd } from "react-icons/md";

import Button from "../../src/components/base/Button";
import { getNavbarLayout } from "../../src/components/layouts/NavbarLayout/NavbarLayout";
import LocalSpaceCard from "../../src/components/ui/LocalSpaceCard";
import { createNewLocalSpace } from "../../src/helpers/indexeddb/LocalSpace/helpers";
import { useLocalSpaces } from "../../src/helpers/indexeddb/LocalSpace/hooks/useLocalSpaces";

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
          <div className="grid grid-cols-3">
            <div />

            <div className="flex justify-center font-black text-3xl">
              Create
            </div>

            <div className="flex justify-end">
              <Button variant="text" squared onClick={handleNewSpace}>
                <div className="flex items-center space-x-1 -ml-0.5">
                  <MdOutlineAdd className="text-lg" />
                  <div>New Scene</div>
                </div>
              </Button>
            </div>
          </div>

          {localSpaces && localSpaces.length === 0 && (
            <div className="flex justify-center space-x-1 text-outline">
              <div>It looks like you don{"'"}t have any scenes.</div>
              <div
                onClick={handleNewSpace}
                className="font-bold hover:underline cursor-pointer"
              >
                Click Here
              </div>
              <div>to create a new scene.</div>
            </div>
          )}

          <div className="w-full grid grid-cols-1 md:grid-cols-2 gap-4">
            {localSpaces?.map((localSpace) => (
              <LocalSpaceCard key={localSpace.id} localSpace={localSpace} />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

Create.getLayout = getNavbarLayout;
