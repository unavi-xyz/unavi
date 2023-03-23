"use client";

import { useSession } from "../../src/client/auth/useSession";
import { useProfileByAddress } from "../../src/play/hooks/useProfileByAddress";
import Avatar from "../../src/ui/Avatar";
import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../src/ui/DropdownMenu";
import ProfileMenu from "./ProfileMenu";

interface Props {
  isLoading?: boolean;
}

export default function ProfileButton({ isLoading: isLoadingAuth }: Props) {
  const { data: session } = useSession();
  const { profile, isLoading: isLoadingProfile } = useProfileByAddress(session?.address);

  const isLoading = isLoadingAuth || isLoadingProfile;

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
        {profile !== undefined && session?.address ? (
          <ProfileMenu profile={profile} session={session} />
        ) : null}
      </DropdownContent>
    </DropdownMenu>
  );
}
