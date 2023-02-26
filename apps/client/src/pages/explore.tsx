import { trpc } from "../client/trpc";
import MetaTags from "../home/MetaTags";
import { getNavbarLayout } from "../home/NavbarLayout/NavbarLayout";
import SpaceCard from "../home/SpaceCard";

export default function Explore() {
  const { data: spaces } = trpc.space.latest.useQuery(
    { limit: 40 },
    {
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    }
  );

  return (
    <>
      <MetaTags title="Explore" />

      <div className="flex justify-center">
        <div className="max-w-content mx-4 space-y-8 py-8">
          <div className="text-center text-3xl font-black">Explore</div>

          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
            {spaces
              ? spaces.map((id) => {
                  return <SpaceCard key={id} id={id} sizes="512" animateEnter />;
                })
              : Array.from({ length: 8 }, (_, i) => (
                  <div
                    key={i}
                    className="aspect-card h-full w-full animate-pulse rounded-xl bg-neutral-300"
                  />
                ))}
          </div>
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
