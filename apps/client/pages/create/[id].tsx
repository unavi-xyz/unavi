import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import Button from "../../src/components/base/Button";

import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";
import { useLocalSpace } from "../../src/helpers/indexeddb/localSpaces/useLocalScene";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const localSpace = useLocalSpace(id);

  if (!localSpace)
    return (
      <div className="flex justify-center mt-8 text-xl">
        Local space not found.
      </div>
    );

  return (
    <div className="flex justify-center my-8">
      <Head>
        <title>{localSpace.name} · The Wired</title>
      </Head>

      <div className="max-w flex space-x-8 h-[500px]">
        <div className="w-full bg-neutral-200 rounded-3xl"></div>
        <div className="w-full flex flex-col justify-between">
          <div className="font-black text-2xl flex justify-center">
            {localSpace.name}
          </div>
          <div></div>

          <Link href={`/studio?id=${localSpace.id}`} passHref>
            <div>
              <Button outline>Open Studio</Button>
            </div>
          </Link>
        </div>
      </div>
    </div>
  );
}

Id.Layout = NavbarLayout;
