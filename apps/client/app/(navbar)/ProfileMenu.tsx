"use client";

import Link from "next/link";
import { MdLogout } from "react-icons/md";

import { useAuth } from "@/src/client/AuthProvider";
import { DropdownItem } from "@/src/ui/DropdownMenu";

interface Props {
  username: string;
}

export default function ProfileMenu({ username }: Props) {
  const { logout } = useAuth();

  return (
    <div className="rounded-xl bg-white py-2 text-lg font-bold">
      <DropdownItem asChild>
        <Link
          href={`/@${username}`}
          draggable={false}
          className="mb-2 flex w-full cursor-pointer items-center whitespace-nowrap px-4 py-1 outline-none focus:bg-neutral-200 active:opacity-80"
        >
          @{username}
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
