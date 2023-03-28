"use client";

import Link from "next/link";
import { MdLogout, MdOutlinePersonOutline, MdOutlineSettings } from "react-icons/md";

import { useLogout } from "../../src/client/auth/useLogout";
import { CustomSession } from "../../src/client/auth/useSession";
import { Profile } from "../../src/server/helpers/fetchProfile";
import { DropdownItem } from "../../src/ui/DropdownMenu";
import { toHex } from "../../src/utils/toHex";

interface Props {
  profile: Profile | null;
  session: CustomSession;
}

export default function ProfileMenu({ profile, session }: Props) {
  const { logout } = useLogout();

  return (
    <div className="py-2 text-lg">
      <DropdownItem asChild>
        <Link
          href={`/user/${profile?.id ? toHex(profile.id) : session?.address}`}
          draggable={false}
          className="flex w-full cursor-pointer items-center whitespace-nowrap py-1 px-4 font-bold outline-none focus:bg-neutral-200 active:opacity-80"
        >
          <MdOutlinePersonOutline className="mr-2 text-xl" />
          <div>Your profile</div>
        </Link>
      </DropdownItem>

      <DropdownItem asChild>
        <Link
          href="/settings"
          draggable={false}
          className="flex w-full cursor-pointer items-center whitespace-nowrap py-1 px-4 font-bold outline-none focus:bg-neutral-200 active:opacity-80"
        >
          <MdOutlineSettings className="mr-2 text-xl" />
          <div>Settings</div>
        </Link>
      </DropdownItem>

      <DropdownItem
        onClick={logout}
        className="flex w-full cursor-pointer items-center whitespace-nowrap py-1 px-4 font-bold outline-none focus:bg-neutral-200 active:opacity-80"
      >
        <MdLogout className="mr-2 text-xl" />
        <div>Sign out</div>
      </DropdownItem>
    </div>
  );
}
