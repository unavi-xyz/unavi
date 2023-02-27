import { fetchLatestSpaces } from "../../../src/server/helpers/fetchLatestSpaces";
import SpaceCard from "./SpaceCard";

export const revalidate = 60;

export const metadata = {
  title: "Explore",
};

export default async function Explore() {
  const spaces = await fetchLatestSpaces(40);

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-8 py-8">
        <div className="text-center text-3xl font-black">Explore</div>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
          {spaces.map(({ id, metadata }) => (
            <SpaceCard key={id} id={id} metadata={metadata} sizes="512" />
          ))}
        </div>
      </div>
    </div>
  );
}
