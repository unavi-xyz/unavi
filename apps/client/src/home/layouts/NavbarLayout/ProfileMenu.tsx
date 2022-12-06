import Link from "next/link";
import { useContext } from "react";
import { HiOutlineSwitchHorizontal } from "react-icons/hi";
import {
  MdLogout,
  MdOutlinePersonOutline,
  MdOutlineSettings,
} from "react-icons/md";

import { LoginContext } from "../../../client/auth/LoginProvider";
import { useLens } from "../../../client/lens/hooks/useLens";
import ProfileMenuButton from "./ProfileMenuButton";

interface Props {
  openSwitchProfile: () => void;
  includeExternal?: boolean;
}

export default function ProfileMenu({
  openSwitchProfile,
  includeExternal = true,
}: Props) {
  const { handle } = useLens();
  const { logout } = useContext(LoginContext);

  if (!handle) return null;

  return (
    <div className="flex flex-col space-y-1 p-2">
      <button onClick={openSwitchProfile} className="w-full">
        <ProfileMenuButton icon={<HiOutlineSwitchHorizontal />}>
          Switch Profile
        </ProfileMenuButton>
      </button>

      {includeExternal && (
        <Link href={`/user/${handle}`}>
          <button className="w-full">
            <ProfileMenuButton icon={<MdOutlinePersonOutline />}>
              Your Profile
            </ProfileMenuButton>
          </button>
        </Link>
      )}

      {includeExternal && (
        <Link href="/settings">
          <button className="w-full">
            <ProfileMenuButton icon={<MdOutlineSettings />}>
              Settings
            </ProfileMenuButton>
          </button>
        </Link>
      )}

      <button onClick={logout} className="w-full">
        <ProfileMenuButton icon={<MdLogout />}>Log Out</ProfileMenuButton>
      </button>
    </div>
  );
}
