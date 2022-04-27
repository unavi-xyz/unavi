import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth, useIpfsFile, useProfile, useSpace } from "ceramic";

import { IconButton } from "../../src/components/base";
import SidebarLayout from "../../src/components/SidebarLayout/SidebarLayout";
import { SpaceSettingsDialog } from "../../src/home/space/SpaceSettingsDialog";

export default function Space() {
  const router = useRouter();
  const id = router.query.id as string;

  const { viewerId } = useAuth();
  const { space, controller } = useSpace(id);
  const { profile } = useProfile(controller);
  const { url } = useIpfsFile(space?.image);

  const [openSettings, setOpenSettings] = useState(false);

  const isOwner = viewerId === controller;

  return (
    <div className="flex flex-col space-y-4 h-full">
      <div className="card flex items-center justify-between">
        <div>
          <div className="text-2xl">{space?.name}</div>
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

            <SpaceSettingsDialog
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
              {space?.description}
            </div>
          </div>

          <div className="h-16">
            <Link href={`/app?space=${id}`} passHref>
              <div
                className="h-full text-md rounded-full bg-black text-white justify-center
                             hover:cursor-pointer transition-all flex items-center"
              >
                Enter Space
              </div>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}

Space.Layout = SidebarLayout;
