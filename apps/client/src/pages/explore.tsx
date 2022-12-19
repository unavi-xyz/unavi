import Link from "next/link";

import { trpc } from "../client/trpc";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import SpaceCard from "../home/lens/SpaceCard";
import MetaTags from "../home/MetaTags";

export default function Explore() {
  const { data: spaces } = trpc.space.latest.useQuery({});

  return (
    <>
      <MetaTags title="Explore" />

      <div className="mx-4 flex justify-center py-8">
        <div className="max-w-content space-y-8">
          <div className="flex justify-center text-3xl font-black">Explore</div>

          <div className="grid grid-cols-3 gap-4">
            {spaces?.map(({ id, metadata }) => {
              return (
                <Link href={`/space/${id}`} key={id}>
                  <SpaceCard metadata={metadata} animateEnter />
                </Link>
              );
            })}
          </div>
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
