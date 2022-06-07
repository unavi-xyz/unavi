import Link from "next/link";
import {
  MdLogout,
  MdOutlinePersonOutline,
  MdOutlineSettings,
} from "react-icons/md";

import { useEthersStore } from "../../../helpers/ethers/store";
import { logout, switchProfile } from "../../../helpers/lens/authentication";
import { useProfilesByAddress } from "../../../helpers/lens/hooks/useProfilesByAddress";
import { useLensStore } from "../../../helpers/lens/store";
import ProfileMenuButton from "./ProfileMenuButton";

export default function ProfileMenu() {
  const handle = useLensStore((state) => state.handle);
  const address = useEthersStore((state) => state.address);
  const profiles = useProfilesByAddress(address);

  const otherProfiles = profiles?.filter(
    (profile) => profile.handle !== handle
  );

  if (!handle) return null;

  return (
    <div className="py-2 space-y-2">
      <div className="gradient-text px-4 font-bold cursor-default select-none">
        @{handle}
      </div>

      <hr />

      <div className="px-2 space-y-1">
        <Link href={`/user/${handle}`} passHref>
          <a className="block">
            <ProfileMenuButton icon={<MdOutlinePersonOutline />}>
              Your Profile
            </ProfileMenuButton>
          </a>
        </Link>

        <Link href="/settings" passHref>
          <a className="block">
            <ProfileMenuButton icon={<MdOutlineSettings />}>
              Settings
            </ProfileMenuButton>
          </a>
        </Link>

        <button onClick={logout} className="w-full">
          <ProfileMenuButton icon={<MdLogout />}>Log Out</ProfileMenuButton>
        </button>
      </div>

      {otherProfiles && otherProfiles.length > 0 && (
        <>
          <hr />

          <div className="px-2 space-y-1 overflow-y-auto max-h-20">
            {otherProfiles.map((profile) => (
              <button
                key={profile.id}
                onClick={() => switchProfile(profile.handle)}
              >
                <ProfileMenuButton>@{profile.handle}</ProfileMenuButton>
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
