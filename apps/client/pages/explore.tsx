import Head from "next/head";
import Link from "next/link";

import { getNavbarLayout } from "../src/components/layouts/NavbarLayout/NavbarLayout";
import SpaceCard from "../src/components/lens/SpaceCard";
import { useExploreSpaces } from "../src/helpers/lens/hooks/useExploreSpaces";

export default function Explore() {
  const spaces = useExploreSpaces();

  return (
    <div>
      <Head>
        <title>Explore / The Wired</title>
      </Head>

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex flex-col items-center justify-center">
            <div className="font-black text-3xl">Explore</div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            {spaces?.map((space) => (
              <Link key={space.id} href={`/space/${space.id}`}>
                <div>
                  <SpaceCard space={space} />
                </div>
              </Link>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

Explore.getLayout = getNavbarLayout;
