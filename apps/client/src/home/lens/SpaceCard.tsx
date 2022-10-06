import { Post } from "@wired-labs/lens";

import Card from "../../ui/Card";
import { getMediaURL } from "../../utils/getMediaURL";

interface Props {
  space: Post;
  sizes?: string;
  animateEnter?: boolean;
}

export default function SpaceCard({ space, sizes, animateEnter }: Props) {
  const image = getMediaURL(space.metadata.media[0]);

  return (
    <Card
      text={space.metadata.name ?? space.id}
      image={image}
      sizes={sizes}
      animateEnter={animateEnter}
    />
  );
}
