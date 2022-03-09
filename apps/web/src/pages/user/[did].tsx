import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useIpfsFile, useProfile, useRooms } from "ceramic";

import { ProfilePicture } from "../../components/base";
import RoomCard from "../../components/RoomCard";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";
import { useState } from "react";
import { ProfileSettingsDialog } from "../../components/ProfileSettingsDialog";

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
    <div className="p-16">
      <ProfileSettingsDialog
        id={did}
        open={openSettings}
        setOpen={setOpenSettings}
      />

      <div className="flex items-center space-x-8">
        <div className="w-32 h-32">
          {image && <ProfilePicture src={image} />}
        </div>

        <div>
          <div className="text-3xl">{profile?.name}</div>
          <div className="text-neutral-400">{did}</div>
        </div>

        {isUser && (
          <div
            onClick={() => setOpenSettings(true)}
            className="py-1.5 px-4 hover:shadow hover:cursor-pointer
                     hover:bg-purple-400 text-xl rounded"
          >
            <IoMdSettings />
          </div>
        )}
      </div>

      <div className="w-full flex justify-end">
        <div className="w-full max-w-2xl space-y-4 p-4 overflow-auto h-[1000px]">
          {rooms?.map((streamId) => {
            return (
              <Link key={streamId} href={`/room/${streamId}`} passHref>
                <div className="hover:cursor-pointer hover:-translate-x-2 transition-all duration-100">
                  <RoomCard streamId={streamId} />
                </div>
              </Link>
            );
          })}
        </div>
      </div>
    </div>
  );
}

User.Layout = SidebarLayout;
