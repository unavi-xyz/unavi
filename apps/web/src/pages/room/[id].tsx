import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useIpfsFile, useRoom } from "ceramic";

import { Button } from "../../components/base";
import { RoomSettingsDialog } from "../../components/RoomSettingsDialog";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";

export default function Room() {
  const router = useRouter();
  const id = router.query.id as string;

  const { viewerId } = useAuth();
  const { room, controller } = useRoom(id);
  const image = useIpfsFile(room?.image);

  const [openSettings, setOpenSettings] = useState(false);

  const isOwner = viewerId === controller;

  return (
    <div className="p-8 space-y-4 max-w-6xl">
      <RoomSettingsDialog
        id={id}
        open={openSettings}
        setOpen={setOpenSettings}
      />

      <div className="flex items-center justify-between">
        <div className="text-3xl">{room?.name}</div>

        {isOwner && (
          <div
            onClick={() => setOpenSettings(true)}
            className="py-1.5 px-4 hover:shadow hover:cursor-pointer
                     hover:bg-indigo-400 text-xl rounded"
          >
            <IoMdSettings />
          </div>
        )}
      </div>

      {image && <img src={image} alt="room image" className="rounded" />}

      <div>{room?.description}</div>

      <Link href={`/app?room=${id}`} passHref>
        <div className="max-w-xs">
          <Button>Join Room</Button>
        </div>
      </Link>
    </div>
  );
}

Room.Layout = SidebarLayout;
