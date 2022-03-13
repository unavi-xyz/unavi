import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useIpfsFile, useProfile, useRoom } from "ceramic";

import { IconButton } from "../../components/base";
import { RoomSettingsDialog } from "../../components/RoomSettingsDialog";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";

export default function Room() {
  const router = useRouter();
  const id = router.query.id as string;

  const { viewerId } = useAuth();
  const { room, controller } = useRoom(id);
  const { profile } = useProfile(controller);
  const image = useIpfsFile(room?.image);

  const [openSettings, setOpenSettings] = useState(false);

  const isOwner = viewerId === controller;

  return (
    <div className="flex flex-col space-y-4 h-2/3">
      <div className="bg-white rounded-3xl shadow p-8 flex items-center justify-between">
        <div>
          <div className="text-2xl">{room?.name}</div>
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

            <RoomSettingsDialog
              id={id}
              open={openSettings}
              setOpen={setOpenSettings}
            />
          </>
        )}
      </div>

      <div className="flex h-full space-x-4">
        <div className="w-3/5 bg-white rounded-3xl shadow">
          {image && (
            <img
              src={image}
              alt="room image"
              className="w-full h-full rounded-3xl object-cover"
            />
          )}
        </div>

        <div className="w-2/5 flex flex-col space-y-4">
          <div className="h-full bg-white rounded-3xl shadow p-8 flex flex-col justify-between">
            <div className="space-y-4">
              <div className="text-2xl flex justify-center">About</div>
              <div className="text-lg">{room?.description}</div>
            </div>

            <div className="h-16">
              <Link href={`/app?room=${id}`} passHref>
                <div
                  className="h-full text-md rounded-full bg-black text-white shadow
                             hover:cursor-pointer hover:bg-opacity-90 transition-all
                             flex items-center justify-center"
                >
                  Join Room
                </div>
              </Link>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

Room.Layout = SidebarLayout;
