import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useIpfsFile, useProfile, useRoom } from "ceramic";

import { IconButton } from "../../components/base";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";
import { RoomSettingsDialog } from "../../components/room/RoomSettingsDialog";

export default function Room() {
  const router = useRouter();
  const id = String(router.query.id);

  const { viewerId } = useAuth();
  const { room, controller } = useRoom(id);
  const { profile } = useProfile(controller);
  const image = useIpfsFile(room?.image);

  const [openSettings, setOpenSettings] = useState(false);

  const isOwner = viewerId === controller;

  return (
    <div className="flex flex-col space-y-4 h-full">
      <div className="card flex items-center justify-between">
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

      <div className="h-full md:h-1/2 md:grid md:grid-cols-3 md:gap-4 space-y-4 md:space-y-0">
        <div className="w-full col-span-2 h-64 md:h-[800px] card-borderless">
          {image && (
            <img
              src={image}
              alt="room image"
              className="w-full h-full rounded-3xl object-cover"
            />
          )}
        </div>

        <div className="card flex flex-col space-y-4">
          <div className="text-2xl font-medium flex justify-center">About</div>

          <div className="relative overflow-auto h-36 md:h-full">
            <div className="absolute top-0 left-0 px-2 whitespace-pre-wrap">
              {room?.description}
            </div>
          </div>

          <div className="h-16">
            <Link href={`/app?room=${id}`} passHref>
              <div
                className="h-full text-md rounded-full bg-black text-white justify-center
                             hover:cursor-pointer transition-all flex items-center"
              >
                Join Room
              </div>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}

Room.Layout = SidebarLayout;
