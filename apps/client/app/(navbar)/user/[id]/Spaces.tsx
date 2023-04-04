import { fetchLatestSpaces } from "@/src/server/helpers/fetchLatestSpaces";
import SpaceCard from "@/src/ui/SpaceCard";

interface Props {
  owner: string;
}

export default async function Spaces({ owner }: Props) {
  const spaces = await fetchLatestSpaces(40, owner);

  return spaces.map(({ id, metadata }) => (
    <SpaceCard
      key={id}
      id={id}
      metadata={metadata}
      sizes="(min-width: 1320px) 33vw, (min-width: 768px) 50vw, 100vw"
    />
  ));
}
