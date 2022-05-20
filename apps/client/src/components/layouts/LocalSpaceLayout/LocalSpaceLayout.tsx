import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";

import { useLocalSpace } from "../../../helpers/indexeddb/LocalSpace/hooks/useLocalSpace";
import Button from "../../base/Button";
import NavbarLayout from "../NavbarLayout/NavbarLayout";
import SpaceTab from "../SpaceLayout/SpaceTab";

interface Props {
  children: React.ReactNode;
}

export default function LocalSpaceLayout({ children }: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const localSpace = useLocalSpace(id);

  if (!localSpace) return null;

  const image = localSpace?.image ?? localSpace?.generatedImage;

  return (
    <>
      <Head>
        <title>{localSpace.name ?? id} · The Wired</title>
      </Head>

      <div className="mx-8 h-full">
        <div className="max-w mx-auto py-8 w-full h-full space-y-8">
          <div className="flex space-x-8">
            <div className="w-full rounded-3xl aspect-card bg-secondaryContainer">
              {image && (
                <img
                  src={URL.createObjectURL(image)}
                  alt="space preview"
                  className="object-cover rounded-3xl w-full h-full"
                />
              )}
            </div>

            <div className="w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="space-y-4">
                <div className="font-black text-3xl flex justify-center">
                  {localSpace.name}
                </div>
              </div>

              <Link href={`/studio/${id}`} passHref>
                <div>
                  <Button variant="filled" fullWidth>
                    <div className="py-2">Open Studio</div>
                  </Button>
                </div>
              </Link>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex space-x-4">
              <SpaceTab href={`/create/${id}`} text="About" />
              <SpaceTab href={`/create/${id}/settings`} text="Settings" />
            </div>

            <div>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}

LocalSpaceLayout.Layout = NavbarLayout;
