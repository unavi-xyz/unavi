import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useIpfsFile, useProfile, useRooms } from "ceramic";

import { IconButton, ProfilePicture } from "../../components/base";
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
    <div className="w-full h-full flex flex-col">
      <div className="flex items-center justify-between bg-white rounded-3xl shadow p-8">
        <div className="flex items-center space-x-8">
          <div className="w-28 h-28">
            {image && <ProfilePicture src={image} />}
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

      <div className="w-full h-full flex space-x-4 pt-4">
        <div className="w-1/2 bg-white rounded-3xl shadow">
          <ProfileAvatar />
        </div>

        <div className="w-1/2 flex flex-col space-y-4">
          <div className="h-1/3 bg-white rounded-3xl shadow p-8 flex flex-col">
            <div className="text-2xl font-medium flex justify-center pb-8">
              About
            </div>

            <div className="h-full">{profile?.description}</div>
          </div>

          <div className="h-2/3 bg-white rounded-3xl shadow flex flex-col">
            <div className="text-2xl font-medium flex justify-center px-8 pt-8">
              Rooms
            </div>

            <div className="space-y-4 h-full overflow-auto p-8">
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
  );
}

User.Layout = SidebarLayout;
