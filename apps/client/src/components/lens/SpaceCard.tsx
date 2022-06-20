import { Post } from "../../generated/graphql";
import Card from "../base/Card";

interface Props {
  space: Post;
}

export default function SpaceCard({ space }: Props) {
  return (
    <Card text={space.metadata.name ?? space.id} image={space.metadata.image} />
  );
}
