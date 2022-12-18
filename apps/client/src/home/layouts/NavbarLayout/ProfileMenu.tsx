import Link from "next/link";
import { MdLogout, MdOutlinePersonOutline, MdOutlineSettings } from "react-icons/md";

import { useLogout } from "../../../client/auth/useLogout";
import { useSession } from "../../../client/auth/useSession";
import { trpc } from "../../../client/trpc";
import { numberToHexDisplay } from "../../../utils/numberToHexDisplay";
import ProfileMenuButton from "./ProfileMenuButton";

interface Props {
  includeExternal?: boolean;
}

export default function ProfileMenu({ includeExternal = true }: Props) {
  const { data: session } = useSession();
  const { logout } = useLogout();

  const { data: profile } = trpc.social.profileByAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  return (
    <div className="space-y-1 p-2">
      {/* <button onClick={openSwitchProfile} className="w-full">
        <ProfileMenuButton icon={<HiOutlineSwitchHorizontal />}>Switch Profile</ProfileMenuButton>
      </button> */}

      {includeExternal && (
        <Link href={`/user/${profile?.id ? numberToHexDisplay(profile.id) : session?.address}`}>
          <button className="w-full">
            <ProfileMenuButton icon={<MdOutlinePersonOutline />}>Your Profile</ProfileMenuButton>
          </button>
        </Link>
      )}

      {includeExternal && (
        <Link href="/settings">
          <button className="w-full">
            <ProfileMenuButton icon={<MdOutlineSettings />}>Settings</ProfileMenuButton>
          </button>
        </Link>
      )}

      <button onClick={logout} className="w-full">
        <ProfileMenuButton icon={<MdLogout />}>Log Out</ProfileMenuButton>
      </button>
    </div>
  );
}
