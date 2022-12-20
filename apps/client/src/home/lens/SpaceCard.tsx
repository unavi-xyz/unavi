import { ERC721Metadata } from "contracts";

import { trpc } from "../../client/trpc";
import Card from "../../ui/Card";

interface Props {
  id: number;
  metadata: ERC721Metadata;
  sizes?: string;
  animateEnter?: boolean;
}

export default function SpaceCard({ id, metadata, sizes, animateEnter }: Props) {
  const { data: playerCount } = trpc.public.playerCount.useQuery({ id });

  return (
    <Card
      text={metadata.name}
      image={metadata.image}
      sizes={sizes}
      animateEnter={animateEnter}
      playerCount={playerCount}
    />
  );
}
