import { Post } from "../../generated/graphql";
import { useMediaImage } from "../../helpers/lens/hooks/useMediaImage";
import Card from "../base/Card";

interface Props {
  avatar: Post;
}

export default function AvatarCard({ avatar }: Props) {
  const image = useMediaImage(avatar.metadata.media[0]);

  return (
    <Card
      text={avatar.metadata.name ?? avatar.id}
      image={image}
      aspect="vertical"
    />
  );
}
