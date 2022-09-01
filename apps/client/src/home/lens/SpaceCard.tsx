import { Post } from "@wired-xr/lens/generated/graphql";

import Card from "../../ui/base/Card";

interface Props {
  space: Post;
}

export default function SpaceCard({ space }: Props) {
  return (
    <Card text={space.metadata.name ?? space.id} image={space.metadata.image} />
  );
}
