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
          <ProfileMenuButton icon={<MdLogout />}>Log Out</ProfileMenuButton>
        </div>
      </div>

      {otherProfiles && otherProfiles.length > 0 && (
        <>
          <hr />

          <div className="px-2 space-y-1 overflow-y-auto max-h-20">
            {otherProfiles.map((profile) => (
              <ProfileMenuButton
                key={profile.id}
                onClick={() => switchProfile(profile.handle)}
              >
                @{profile.handle}
              </ProfileMenuButton>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
