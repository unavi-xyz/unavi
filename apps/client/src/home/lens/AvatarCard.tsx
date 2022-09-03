import { Post } from "@wired-labs/lens";

import Card from "../../ui/base/Card";

interface Props {
  avatar: Post;
  sizes?: string;
}

export default function AvatarCard({ avatar, sizes }: Props) {
  return (
    <Card
      text={avatar.metadata.name ?? avatar.id}
      image={avatar.metadata.image}
      aspect="vertical"
      sizes={sizes}
    />
  );
}
