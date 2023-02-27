import Link from "next/link";
import { MdPeople } from "react-icons/md";

import { fetchPlayerCount } from "../../../src/server/helpers/fetchPlayerCount";
import { fetchSpace } from "../../../src/server/helpers/fetchSpace";
import Card from "../../../src/ui/Card";
import { numberToHexDisplay } from "../../../src/utils/numberToHexDisplay";

interface Props {
  id: number;
  sizes?: string;
}

export default async function SpaceCard({ id, sizes }: Props) {
  const [space, playerCount] = await Promise.all([fetchSpace(id), fetchPlayerCount(id)]);

  if (!space.metadata) return null;

  return (
    <Link href={`/space/${numberToHexDisplay(id)}`} className="rounded-xl">
      <Card text={space.metadata?.name} image={space.metadata?.image} sizes={sizes}>
        {playerCount > 0 && (
          <div className="absolute flex h-full w-full items-start p-2 tracking-wide">
            <div className="flex items-center space-x-1.5 rounded-full bg-black/50 px-3 py-0.5 text-white  backdrop-blur-lg">
              <MdPeople className="text-lg" />
              <div className="font-bold">{playerCount}</div>
            </div>
          </div>
        )}
      </Card>
    </Link>
  );
}
