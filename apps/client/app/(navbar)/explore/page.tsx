import { Space__factory, SPACE_ADDRESS } from "contracts";
import { Suspense } from "react";

import { ethersProvider } from "../../../src/server/constants";
import Card from "../../../src/ui/Card";
import SpaceCard from "./SpaceCard";

const LIMIT = 40;

export const metadata = {
  title: "Explore",
};

export default async function Explore() {
  const spaceContract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

  const count = (await spaceContract.count()).toNumber();

  const isSpaceValid = async (tokenId: number) => {
    try {
      // Check if uri exists
      const uri = await spaceContract.tokenURI(tokenId);
      if (uri) return tokenId;
    } catch {
      // Ignore
    }
  };

  type ValidResponse = Exclude<Awaited<ReturnType<typeof isSpaceValid>>, undefined>;
  const spaces: ValidResponse[] = [];
  let nextSpaceId = count - 1;

  const fetchSpace = async () => {
    if (nextSpaceId === 0 || spaces.length === LIMIT) return;

    const valid = await isSpaceValid(nextSpaceId--);

    if (valid) spaces.push(valid);
    else await fetchSpace();
  };

  const amountToFetch = Math.min(LIMIT, count);
  await Promise.all(Array.from({ length: amountToFetch }).map(fetchSpace));
  const sortedSpaces = spaces.sort((a, b) => b - a);

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-8 py-8">
        <div className="text-center text-3xl font-black">Explore</div>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
          {sortedSpaces.map((id) => {
            return (
              <Suspense key={id} fallback={<Card loading />}>
                {/* @ts-expect-error Server Component */}
                <SpaceCard id={id} sizes="512" />
              </Suspense>
            );
          })}
        </div>
      </div>
    </div>
  );
}
