import { PostFieldsFragment } from "../../generated/graphql";
import { useMediaImage } from "../../helpers/lens/hooks/useMediaImage";

import Card from "../base/Card";

interface Props {
  space: PostFieldsFragment;
}

export default function SpaceCard({ space }: Props) {
  const { url } = useMediaImage(space.metadata.media[0]);

  return <Card text={space?.metadata?.name ?? ""} image={url} />;
}
