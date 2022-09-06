import Link from "next/link";
import { useContext } from "react";
import {
  MdLogout,
  MdOutlinePersonOutline,
  MdOutlineSettings,
} from "react-icons/md";

import { LoginContext } from "../../../auth/LoginProvider";
import { useLens } from "../../../lib/lens/hooks/useLens";
import ProfileMenuButton from "./ProfileMenuButton";

interface Props {
  openSwitchProfile: () => void;
}

export default function ProfileMenu({ openSwitchProfile }: Props) {
  const { handle } = useLens();
  const { logout } = useContext(LoginContext);

  if (!handle) return null;

  return (
    <div className="p-2 space-y-1">
      <button
        onClick={openSwitchProfile}
        className="flex justify-center rounded-lg
                     w-full py-1 px-5 space-x-2 transition font-bold
                     hover:bg-primaryContainer hover:text-onPrimaryContainer"
      >
        @{handle}
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
