import { ERC721Metadata } from "contracts";

import Card from "../../ui/Card";

interface Props {
  metadata: ERC721Metadata;
  sizes?: string;
  animateEnter?: boolean;
}

export default function SpaceCard({ metadata, sizes, animateEnter }: Props) {
  // const image = metadata.media ? getMediaURL(metadata.media[0]) : null;

  // const { data: playerCount } = trpc.public.playerCount.useQuery({
  //   id: space.id,
  // });

  return (
    <Card
      text={metadata.name}
      image={metadata.image}
      sizes={sizes}
      animateEnter={animateEnter}
      // playerCount={playerCount}
    />
  );
}
