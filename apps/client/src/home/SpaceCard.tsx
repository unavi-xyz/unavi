import Link from "next/link";
import { MdPeople } from "react-icons/md";

import { trpc } from "../client/trpc";
import Card from "../ui/Card";
import { numberToHexDisplay } from "../utils/numberToHexDisplay";

interface Props {
  id: number;
  sizes?: string;
  animateEnter?: boolean;
}

export default function SpaceCard({ id, sizes, animateEnter }: Props) {
  const { data: space, isLoading } = trpc.space.byId.useQuery({ id });
  const { data: playerCount } = trpc.public.playerCount.useQuery({ id });

  if (!isLoading && !space?.metadata) return null;

  return (
    <Link href={`/space/${numberToHexDisplay(id)}`} className="rounded-xl">
      <Card
        text={space?.metadata?.name}
        image={space?.metadata?.image}
        sizes={sizes}
        animateEnter={animateEnter}
        loading={isLoading}
      >
        <div className="absolute flex h-full w-full items-start p-2 tracking-wide">
          {playerCount !== undefined && playerCount > 0 && (
            <div className="flex items-center space-x-1.5 rounded-full bg-black/50 px-3 py-0.5 text-white  backdrop-blur-lg">
              <MdPeople className="text-lg" />
              <div className="font-bold">{playerCount}</div>
            </div>
          )}
        </div>
      </Card>
    </Link>
  );
}
