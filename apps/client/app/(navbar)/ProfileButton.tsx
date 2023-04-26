"use client";

import { useState } from "react";

import { useAuth } from "@/src/client/AuthProvider";
import { useProfileByAddress } from "@/src/play/hooks/useProfileByAddress";
import Avatar from "@/src/ui/Avatar";
import { DropdownContent, DropdownMenu, DropdownTrigger } from "@/src/ui/DropdownMenu";

import ProfileMenu from "./ProfileMenu";

interface Props {
  loading?: boolean;
}

export default function ProfileButton({ loading: loadingAuth }: Props) {
  const { user } = useAuth();
  const { profile, isLoading: loadingProfile } = useProfileByAddress(user?.address);

  const [open, setOpen] = useState(false);

  const loading = loadingAuth || loadingProfile;

  return (
    <DropdownMenu
      open={open}
      onOpenChange={(value) => {
        if (loading && value) return;
        setOpen(value);
      }}
    >
      <DropdownTrigger
        className={`rounded-full transition ${loading ? "cursor-default" : "hover:opacity-90"}`}
      >
        <Avatar
          src={profile?.metadata?.image}
          uniqueKey={profile?.handle?.full ?? user?.address ?? ""}
          loading={loading}
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
