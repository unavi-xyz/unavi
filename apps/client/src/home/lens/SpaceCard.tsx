import { Post } from "lens";

import { trpc } from "../../client/trpc";
import Card from "../../ui/Card";
import { getMediaURL } from "../../utils/getMediaURL";

interface Props {
  space: Post;
  sizes?: string;
  animateEnter?: boolean;
}

export default function SpaceCard({ space, sizes, animateEnter }: Props) {
  const image = getMediaURL(space.metadata.media[0]);

  const { data: playerCount } = trpc.public.playerCount.useQuery({
    id: space.id,
  });

  return (
    <Card
      text={space.metadata.name ?? space.id}
      image={image}
      sizes={sizes}
      animateEnter={animateEnter}
      playerCount={playerCount}
    />
  );
}
