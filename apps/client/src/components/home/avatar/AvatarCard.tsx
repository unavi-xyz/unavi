import { useAvatar, useIpfsImage } from "ceramic";
import { Card } from "../../base";

interface Props {
  id: string;
}

export default function AvatarCard({ id }: Props) {
  const { avatar } = useAvatar(id);
  const image = useIpfsImage(avatar?.image);

  return <Card text={avatar?.name} image={image} />;
}
