import Link from "next/link";
import {
  MdLogout,
  MdOutlinePersonOutline,
  MdOutlineSettings,
} from "react-icons/md";

import { logout } from "../../../helpers/lens/authentication";
import { useLensStore } from "../../../helpers/lens/store";
import ProfileMenuButton from "./ProfileMenuButton";

export default function ProfileMenu() {
  const handle = useLensStore((state) => state.handle);

  if (!handle) return null;

  return (
    <div className="py-2 space-y-2">
      <div className="px-2">
        <div className="gradient-text px-2 font-bold cursor-default select-none">
          @{handle}
        </div>
      </div>

      <div className="px-2 space-y-2">
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

        <div onClick={logout}>
          <ProfileMenuButton icon={<MdLogout />}>Log Out </ProfileMenuButton>
        </div>
      </div>
    </div>
  );
}
