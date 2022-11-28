import { Post } from "lens";

import Card from "../../ui/Card";
import { getMediaURL } from "../../utils/getMediaURL";

interface Props {
  avatar: Post;
  sizes?: string;
  animateEnter?: boolean;
}

export default function AvatarCard({ avatar, sizes, animateEnter }: Props) {
  const image = getMediaURL(avatar.metadata.media[0]);

  return (
    <Card
      text={avatar.metadata.name ?? avatar.id}
      image={image}
      aspect="vertical"
      sizes={sizes}
      animateEnter={animateEnter}
    />
  );
}
