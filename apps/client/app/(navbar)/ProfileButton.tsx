"use client";

import { User } from "lucia-auth";
import { useState } from "react";

import Avatar from "@/src/ui/Avatar";
import {
  DropdownContent,
  DropdownMenu,
  DropdownTrigger,
} from "@/src/ui/DropdownMenu";

import ProfileMenu from "./ProfileMenu";

interface Props {
  user: User;
  image?: string | null;
  loading?: boolean;
}

export default function ProfileButton({ user, image, loading }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownTrigger className="rounded-full transition hover:opacity-90">
        <Avatar
          src={image}
          uniqueKey={user.username}
          circle
          size={40}
          loading={loading}
        />
      </DropdownTrigger>

      <DropdownContent>
        <ProfileMenu user={user} />
      </DropdownContent>
    </DropdownMenu>
  );
}
