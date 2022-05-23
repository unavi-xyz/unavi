import { Post } from "../../generated/graphql";
import { useMediaImage } from "../../helpers/lens/hooks/useMediaImage";
import Card from "../base/Card";

interface Props {
  space: Post;
}

export default function SpaceCard({ space }: Props) {
  const image = useMediaImage(space.metadata.media[0]);

  return <Card text={space.metadata.name ?? space.id} image={image} />;
}
