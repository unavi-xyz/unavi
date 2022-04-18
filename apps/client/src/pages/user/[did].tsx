import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useIpfsFile, useProfile, useUserSpaces } from "ceramic";

import { IconButton } from "../../components/base";
import { ProfileSettingsDialog } from "../../components/home/profile/ProfileSettingsDialog";

import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";
import ProfileAvatar from "../../components/home/profile/ProfileAvatar";
import SpaceCard from "../../components/home/space/SpaceCard";

export default function User() {
  const router = useRouter();
  const did = router.query.did as string;

  const { viewerId } = useAuth();
  const { profile } = useProfile(did);
  const { url } = useIpfsFile(profile?.image?.original.src);
  const spaces = useUserSpaces(did);

  const [openSettings, setOpenSettings] = useState(false);

  const isUser = viewerId === did;

  return (
    <div className="w-full h-full space-y-4 flex flex-col">
      <div className="card flex items-center justify-between">
        <div className="flex items-center space-x-8">
          <div className="w-28 h-28 flex-shrink-0">
            {url && (
              <img
                src={url}
                alt="profile picture"
                className="object-cover rounded-full w-full h-full"
              />
            )}
          </div>

          <div>
            <div className="text-3xl">{profile?.name}</div>
            <div className="text-neutral-400 break-all">{did}</div>
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
              id={did}
              open={openSettings}
              setOpen={setOpenSettings}
            />
          </>
        )}
      </div>

      <div className="basis-full grid md:grid-cols-2 gap-4 pb-4 md:pb-0">
        <div className="card-borderless h-96 md:h-full">
          <ProfileAvatar avatarId={profile?.avatar} />
        </div>

        <div className="grid md:grid-rows-3 gap-4">
          <div className="card flex flex-col space-y-4">
            <div className="text-2xl font-medium flex justify-center">
              About
            </div>

            <div className="relative overflow-auto h-96 md:h-full">
              <div className="absolute top-0 left-0 px-2 whitespace-pre-wrap">
                {profile?.description}
              </div>
            </div>
          </div>

          <div className="card row-span-2 flex flex-col space-y-4">
            <div className="text-2xl font-medium flex justify-center">
              Spaces
            </div>

            <div className="relative overflow-auto h-96 md:h-full">
              <div className="absolute h-full w-full top-0 left-0 px-4 pb-4 space-x-8 flex">
                {spaces?.map((streamId) => {
                  return (
                    <Link key={streamId} href={`/space/${streamId}`} passHref>
                      <div className="flex-shrink-0 float-left w-96 h-full">
                        <SpaceCard streamId={streamId} />
                      </div>
                    </Link>
                  );
                })}

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
