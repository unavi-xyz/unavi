import { fetchPlayerCount } from "@/src/server/helpers/fetchPlayerCount";

interface Props {
  id: number;
}

export default async function PlayerCount({ id }: Props) {
  const playerCount = await fetchPlayerCount(id);

  if (playerCount === 0) return null;

  return (
    <div className="flex justify-center space-x-1 font-bold md:justify-start">
      <div>{playerCount}</div>
      <div className="text-neutral-500">connected player{playerCount === 1 ? null : "s"}</div>
    </div>
  );
}
