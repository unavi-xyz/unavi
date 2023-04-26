"use client";

import { User } from "lucia-auth";
import { useState } from "react";

import { Profile } from "@/src/server/helpers/fetchProfile";
import Avatar from "@/src/ui/Avatar";
import { DropdownContent, DropdownMenu, DropdownTrigger } from "@/src/ui/DropdownMenu";

import ProfileMenu from "./ProfileMenu";

interface Props {
  user: User;
  profile: Profile | null;
}

export default function ProfileButton({ user, profile }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownTrigger className="rounded-full transition hover:opacity-90">
        <Avatar
          src={profile?.metadata?.image}
          uniqueKey={profile?.handle?.full ?? user.address}
          circle
          size={40}
        />
      </DropdownTrigger>

      <DropdownContent>
        {profile !== undefined && user?.address ? (
          <ProfileMenu profile={profile} user={user} />
        ) : null}
      </DropdownContent>
    </DropdownMenu>
  );
}
