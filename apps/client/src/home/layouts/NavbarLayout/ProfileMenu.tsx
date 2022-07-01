import Link from "next/link";
import { useContext } from "react";
import {
  MdLogout,
  MdOutlinePersonOutline,
  MdOutlineSettings,
} from "react-icons/md";

import { EthersContext } from "@wired-xr/ethers";
import { LensContext, trimHandle, useProfilesByAddress } from "@wired-xr/lens";

import ProfileMenuButton from "./ProfileMenuButton";

export default function ProfileMenu() {
  const { handle, logout, switchProfile } = useContext(LensContext);
  const { address } = useContext(EthersContext);

  const profiles = useProfilesByAddress(address);

  const otherProfiles = profiles?.filter(
    (profile) => trimHandle(profile.handle) !== handle
  );

  if (!handle) return null;

  return (
    <div className="py-2 space-y-2">
      <div className="px-6 font-bold" onPointerUp={(e) => e.stopPropagation()}>
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

          <div
            onPointerUp={(e) => e.stopPropagation()}
            className="flex justify-center font-bold"
          >
            Switch Profiles
          </div>

          <div className="px-2 space-y-1 overflow-y-auto max-h-20 flex flex-col">
            {otherProfiles.map((profile) => {
              const profileHandle = trimHandle(profile.handle);
              return (
                <button
                  key={profile.id}
                  onClick={() => switchProfile(profileHandle)}
                >
                  <ProfileMenuButton>@{profileHandle}</ProfileMenuButton>
                </button>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
