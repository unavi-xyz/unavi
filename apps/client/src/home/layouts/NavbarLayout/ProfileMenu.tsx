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
}

export default function ProfileMenu({ openSwitchProfile }: Props) {
  const { handle } = useLens();
  const { logout } = useContext(LoginContext);

  if (!handle) return null;

  return (
    <div className="space-y-1 p-2">
      <button onClick={openSwitchProfile} className="w-full">
        <ProfileMenuButton icon={<HiOutlineSwitchHorizontal />}>
          Switch Profile
        </ProfileMenuButton>
      </button>

      <Link href={`/user/${handle}`} passHref>
        <div>
          <ProfileMenuButton icon={<MdOutlinePersonOutline />}>
            Your Profile
          </ProfileMenuButton>
        </div>
      </Link>

      <Link href="/settings" passHref>
        <div>
          <ProfileMenuButton icon={<MdOutlineSettings />}>
            Settings
          </ProfileMenuButton>
        </div>
      </Link>

      <button onClick={logout} className="w-full">
        <ProfileMenuButton icon={<MdLogout />}>Log Out</ProfileMenuButton>
      </button>
    </div>
  );
}
