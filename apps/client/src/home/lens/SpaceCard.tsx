import { Post } from "@wired-xr/lens";

import Card from "../../ui/base/Card";

interface Props {
  space: Post;
  sizes?: string;
}

export default function SpaceCard({ space, sizes }: Props) {
  return (
    <Card
      text={space.metadata.name ?? space.id}
      image={space.metadata.image}
      sizes={sizes}
    />
  );
}
