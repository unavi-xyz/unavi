import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useIpfsFile, useProfile, useRooms } from "ceramic";

import { IconButton } from "../../components/base";
import { ProfileSettingsDialog } from "../../components/ProfileSettingsDialog";
import RoomCard from "../../components/RoomCard";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";
import ProfileAvatar from "../../components/ProfileAvatar";

export default function User() {
  const router = useRouter();
  const did = router.query.did as string;

  const { viewerId } = useAuth();
  const { profile } = useProfile(did);
  const image = useIpfsFile(profile?.image?.original.src);
  const rooms = useRooms(did);

  const [openSettings, setOpenSettings] = useState(false);

  const isUser = viewerId === did;

  return (
    <div className="w-full h-full space-y-4 flex flex-col">
      <div className="card flex items-center justify-between">
        <div className="flex items-center space-x-8">
          <div className="w-28 h-28">
            {image && (
              <img
                src={image}
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
            <div className="mr-8">
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

      <div className="basis-full grid grid-cols-2 gap-4">
        <div className="bg-white rounded-3xl shadow-sm">
          <ProfileAvatar />
        </div>

        <div className="grid grid-rows-3 gap-4">
          <div className="card flex flex-col space-y-4">
            <div className="text-2xl font-medium flex justify-center">
              About
            </div>

            <div className="relative h-full overflow-auto">
              <div className="absolute top-0 left-0 px-2 whitespace-pre-wrap">
                {profile?.description}
              </div>
            </div>
          </div>

          <div className="card row-span-2 flex flex-col space-y-4">
            <div className="text-2xl font-medium flex justify-center">
              Rooms
            </div>

            <div className="relative h-full overflow-auto">
              <div className="absolute w-full top-0 left-0 px-2 space-y-4">
                {rooms?.map((streamId) => {
                  return (
                    <Link key={streamId} href={`/room/${streamId}`} passHref>
                      <div>
                        <RoomCard streamId={streamId} />
                      </div>
                    </Link>
                  );
                })}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

User.Layout = SidebarLayout;
