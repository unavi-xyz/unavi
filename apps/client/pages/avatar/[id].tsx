import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useProfile, useAvatar, useIpfsFile } from "ceramic";
import { useQueryClient } from "react-query";

import { IconButton } from "../../src/components/base";
import SidebarLayout from "../../src/components/SidebarLayout/SidebarLayout";
import { AvatarSettingsDialog } from "../../src/home/avatar/AvatarSettingsDialog";

export default function Avatar() {
  const router = useRouter();
  const id = router.query.id as string;

  const queryClient = useQueryClient();
  const { viewerId, authenticated } = useAuth();
  const { avatar, controller } = useAvatar(id);
  const { profile } = useProfile(controller);
  const { profile: userProfile, merge } = useProfile(viewerId);
  const { url } = useIpfsFile(avatar?.image);

  const [openSettings, setOpenSettings] = useState(false);

  const isOwner = viewerId === controller;

  async function handleEquip() {
    await merge({ avatar: id });
    queryClient.invalidateQueries(`basicProfile-${viewerId}`);
  }

  const isEquipped = userProfile?.avatar === id;

  return (
    <div className="flex flex-col space-y-4 h-full">
      <div className="card flex items-center justify-between">
        <div>
          <div className="text-2xl">{avatar?.name}</div>
          <div className="text-lg text-neutral-500">
            by{" "}
            <Link href={`/user/${controller}`} passHref>
              <span className="hover:cursor-pointer hover:underline">
                {profile?.name ?? controller}
              </span>
            </Link>
          </div>
        </div>

        {isOwner && (
          <>
            <IconButton onClick={() => setOpenSettings(true)}>
              <IoMdSettings />
            </IconButton>

            <AvatarSettingsDialog
              id={id}
              open={openSettings}
              setOpen={setOpenSettings}
            />
          </>
        )}
      </div>

      <div className="h-full md:h-1/2 md:grid md:grid-cols-3 md:gap-4 space-y-4 md:space-y-0">
        <div className="w-full col-span-2 h-64 md:h-[800px] card-borderless">
          {url && (
            <img
              src={url}
              alt="space image"
              className="w-full h-full rounded-3xl object-cover"
            />
          )}
        </div>

        <div className="card flex flex-col space-y-4">
          <div className="text-2xl font-medium flex justify-center">About</div>

          <div className="relative overflow-auto h-36 md:h-full">
            <div className="absolute top-0 left-0 px-2 whitespace-pre-wrap">
              {avatar?.description}
            </div>
          </div>

          {authenticated && (
            <div className="h-16">
              {isEquipped ? (
                <div
                  className="h-full text-md rounded-full bg-white text-black border border-black
                           flex items-center justify-center cursor-not-allowed"
                >
                  Currently Equipped
                </div>
              ) : (
                <div
                  onClick={handleEquip}
                  className="h-full text-md rounded-full bg-black text-white justify-center
                             hover:cursor-pointer flex items-center"
                >
                  Equip Avatar
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

Avatar.Layout = SidebarLayout;
