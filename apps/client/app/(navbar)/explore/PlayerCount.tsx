import { MdPeople } from "react-icons/md";

import { fetchPlayerCount } from "../../../src/server/helpers/fetchPlayerCount";

interface Props {
  id: number;
}

export default async function PlayerCount({ id }: Props) {
  const playerCount = await fetchPlayerCount(id);

  if (playerCount === 0) return null;

  return (
    <div className="absolute flex h-full w-full items-start p-2 tracking-wide">
      <div className="flex items-center space-x-1.5 rounded-full bg-black/50 px-3 py-0.5 text-white  backdrop-blur-lg">
        <MdPeople className="text-lg" />
        <div className="font-bold">{playerCount}</div>
      </div>
    </div>
  );
}
