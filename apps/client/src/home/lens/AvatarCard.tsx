import { Post } from "@wired-xr/lens";

import Card from "../../ui/base/Card";

interface Props {
  avatar: Post;
}

export default function AvatarCard({ avatar }: Props) {
  return (
    <Card
      text={avatar.metadata.name ?? avatar.id}
      image={avatar.metadata.image}
      aspect="vertical"
    />
  );
}
