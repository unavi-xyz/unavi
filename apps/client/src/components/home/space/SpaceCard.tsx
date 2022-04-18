import { useIpfsFile, useSpace } from "ceramic";
import { Card } from "../../base";

interface Props {
  streamId: string;
}

export default function SpaceCard({ streamId }: Props) {
  const { space } = useSpace(streamId);
  const { url } = useIpfsFile(space?.image);

  if (!space) return null;

  return <Card image={url} text={space?.name} />;
}
