"use client";

import { useState } from "react";

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

  const [open, setOpen] = useState(false);

  const isLoading = isLoadingAuth || isLoadingProfile;

  return (
    <DropdownMenu
      open={open}
      onOpenChange={(value) => {
        if (isLoading && value) return;
        setOpen(value);
      }}
    >
      <DropdownTrigger
        className={`rounded-full transition ${isLoading ? "cursor-default" : "hover:opacity-90"}`}
      >
        <Avatar
          src={profile?.metadata?.image}
          uniqueKey={profile?.handle?.full ?? session?.address ?? ""}
          loading={isLoading}
          circle
          size={40}
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
