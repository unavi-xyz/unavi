"use client";

import { CustomSession } from "../../src/client/auth/useSession";
import Avatar from "../../src/home/Avatar";
import { useProfileByAddress } from "../../src/play/hooks/useProfileByAddress";
import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../src/ui/DropdownMenu";
import ProfileMenu from "./ProfileMenu";

interface Props {
  session: CustomSession;
}

export default function ProfileButton({ session }: Props) {
  if (!session.address) throw new Error("No address found");

  const { profile, isLoading } = useProfileByAddress(session.address);

  return (
    <DropdownMenu>
      <DropdownTrigger className="rounded-full">
        <Avatar
          src={profile?.metadata?.image}
          uniqueKey={profile?.handle?.full ?? session?.address ?? ""}
          loading={isLoading}
          circle
          size={36}
        />
      </DropdownTrigger>

      <DropdownContent>
        {profile !== undefined && <ProfileMenu profile={profile} session={session} />}
      </DropdownContent>
    </DropdownMenu>
  );
}
