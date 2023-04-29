"use client";

import { User } from "lucia-auth";
import Link from "next/link";
import { MdLogout } from "react-icons/md";

import { useAuth } from "@/src/client/AuthProvider";
import { DropdownItem } from "@/src/ui/DropdownMenu";

interface Props {
  user: User;
}

export default function ProfileMenu({ user }: Props) {
  const { logout } = useAuth();

  return (
    <div className="py-2 text-lg font-bold">
      <DropdownItem asChild>
        <Link
          href={`/@${user.username}`}
          draggable={false}
          className="mb-2 flex w-full cursor-pointer items-center whitespace-nowrap px-4 py-1 outline-none focus:bg-neutral-200 active:opacity-80"
        >
          @{user.username}
        </Link>
      </DropdownItem>

      <hr className="pb-2" />

      <DropdownItem
        onClick={logout}
        className="flex w-full cursor-pointer items-center whitespace-nowrap py-1 pl-4 pr-5 outline-none focus:bg-neutral-200 active:opacity-80"
      >
        <MdLogout className="mr-2 text-xl" />
        <div>Sign out</div>
      </DropdownItem>
    </div>
  );
}
