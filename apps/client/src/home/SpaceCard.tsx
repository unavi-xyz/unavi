import { ERC721Metadata } from "contracts";
import { MdPeople } from "react-icons/md";

import { trpc } from "../client/trpc";
import Card from "../ui/Card";

interface Props {
  id: number;
  metadata: ERC721Metadata;
  sizes?: string;
  animateEnter?: boolean;
}

export default function SpaceCard({ id, metadata, sizes, animateEnter }: Props) {
  const { data: playerCount } = trpc.public.playerCount.useQuery({ id });

  return (
    <Card text={metadata.name} image={metadata.image} sizes={sizes} animateEnter={animateEnter}>
      <div className="absolute flex h-full w-full items-start p-2 tracking-wide">
        {playerCount !== undefined && playerCount > 0 && (
          <div className="flex items-center space-x-1.5 rounded-full bg-black/50 px-3 py-0.5 text-white  backdrop-blur-lg">
            <MdPeople className="text-lg" />
            <div className="font-bold">{playerCount}</div>
          </div>
        )}
      </div>
    </Card>
  );
}
