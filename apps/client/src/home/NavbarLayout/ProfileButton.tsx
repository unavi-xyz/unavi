"use client";

import { useState } from "react";

import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../ui/DropdownMenu";
import Avatar from "../Avatar";
import ProfileMenu from "./ProfileMenu";

export default function ProfileButton() {
  const [open, setOpen] = useState(false);
  const { data: session, status } = useSession();

  const { data: profile, isLoading: isProfileLoading } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  const isLoading = status === "loading" || isProfileLoading;

  return (
    <DropdownMenu
      open={open}
      onOpenChange={(value) => {
        if (isLoading) setOpen(false);
        else setOpen(value);
      }}
    >
      <DropdownTrigger disabled={isLoading} className="rounded-full">
        <Avatar
          src={profile?.metadata?.image}
          uniqueKey={profile?.handle?.full ?? session?.address ?? ""}
          loading={isLoading}
          circle
          size={36}
        />
      </DropdownTrigger>

      <DropdownContent open={open}>
        <ProfileMenu />
      </DropdownContent>
    </DropdownMenu>
  );
}
