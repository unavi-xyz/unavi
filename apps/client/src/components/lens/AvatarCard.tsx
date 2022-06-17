import { Post } from "../../generated/graphql";
import Card from "../base/Card";

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
