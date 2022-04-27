import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";

import { IconButton } from "../../src/components/base";
import { ProfileSettingsDialog } from "../../src/home/profile/ProfileSettingsDialog";
import { useProfileByHandle } from "../../src/helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../src/helpers/lens/store";

import SidebarLayout from "../../src/components/SidebarLayout/SidebarLayout";
import ProfileAvatar from "../../src/home/profile/ProfileAvatar";
import SpaceCard from "../../src/home/space/SpaceCard";
import useProfilePicture from "../../src/helpers/lens/hooks/useProfilePicture";
import { useColorFromSeed } from "../../src/helpers/hooks/useColorFromSeed";

export default function User() {
  const router = useRouter();
  const handle = router.query.handle as string;

  const viewerHandle = useLensStore((state) => state.handle);
  const [openSettings, setOpenSettings] = useState(false);

  const { data } = useProfileByHandle(handle);
  const profile = data?.profiles.items[0];
  const { url } = useProfilePicture(profile?.picture);
  const color = useColorFromSeed(handle);

  const isUser = viewerHandle && handle && viewerHandle === handle;

  return (
    <div className="w-full h-full space-y-4 flex flex-col">
      <div className="card flex items-center justify-between">
        <div className="flex items-center space-x-8">
          <div className="w-28 h-28 flex-shrink-0">
            {url ? (
              <img
                src={url}
                alt="profile picture"
                className="object-cover rounded-full w-full h-full"
              />
            ) : (
              <div
                style={{ backgroundColor: color }}
                className="object-cover rounded-full w-full h-full"
              />
            )}
          </div>

          <div>
            <div className="text-3xl">{profile?.name ?? handle}</div>
            <div className="text-neutral-400 break-all">@{handle}</div>
          </div>
        </div>

        {isUser && (
          <>
            <div className="md:mr-8">
              <IconButton onClick={() => setOpenSettings(true)}>
                <IoMdSettings />
              </IconButton>
            </div>

            <ProfileSettingsDialog
              handle={handle}
              open={openSettings}
              setOpen={setOpenSettings}
            />
          </>
        )}
      </div>

      <div className="basis-full grid md:grid-cols-2 gap-4 pb-4 md:pb-0">
        <div className="card-borderless h-96 md:h-full">
          {/* <ProfileAvatar avatarId={profile?.avatar} /> */}
        </div>

        <div className="grid md:grid-rows-3 gap-4">
          <div className="card flex flex-col space-y-4">
            <div className="text-2xl font-medium flex justify-center">
              About
            </div>

            <div className="relative overflow-auto h-96 md:h-full">
              <div className="absolute top-0 left-0 px-2 whitespace-pre-wrap">
                {profile?.bio}
              </div>
            </div>
          </div>

          <div className="card row-span-2 flex flex-col space-y-4">
            <div className="text-2xl font-medium flex justify-center">
              Spaces
            </div>

            <div className="relative overflow-auto h-96 md:h-full">
              <div className="absolute h-full w-full top-0 left-0 px-4 pb-4 space-x-8 flex">
                {/* {spaces?.map((streamId) => {
                  return (
                    <Link key={streamId} href={`/space/${streamId}`} passHref>
                      <div className="flex-shrink-0 float-left w-96 h-full">
                        <SpaceCard streamId={streamId} />
                      </div>
                    </Link>
                  );
                })} */}

                <div className="pr-2" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

User.Layout = SidebarLayout;
