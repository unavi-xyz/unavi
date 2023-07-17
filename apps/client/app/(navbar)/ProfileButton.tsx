import Avatar from "@/src/ui/Avatar";
import {
  DropdownContent,
  DropdownMenu,
  DropdownTrigger,
} from "@/src/ui/DropdownMenu";

import ProfileMenu from "./ProfileMenu";

interface Props {
  username: string;
  image?: string | null;
  loading?: boolean;
}

export default function ProfileButton({ username, image, loading }: Props) {
  return (
    <DropdownMenu>
      <DropdownTrigger className="rounded-full transition hover:opacity-90">
        <Avatar
          src={image}
          uniqueKey={username}
          circle
          size={40}
          loading={loading}
        />
      </DropdownTrigger>

      <DropdownContent>
        <ProfileMenu username={username} />
      </DropdownContent>
    </DropdownMenu>
  );
}
