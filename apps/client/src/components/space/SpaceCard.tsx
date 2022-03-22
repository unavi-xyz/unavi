import { useIpfsImage, useSpace } from "ceramic";
import { Card } from "../base";

interface Props {
  streamId: string;
}

export default function SpaceCard({ streamId }: Props) {
  const { space } = useSpace(streamId);
  const image = useIpfsImage(space?.image);

  if (!space) return null;

  return <Card image={image} text={space?.name} />;
}
