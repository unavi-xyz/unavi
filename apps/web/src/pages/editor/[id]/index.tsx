import { useState } from "react";
import { FaHammer } from "react-icons/fa";
import { IoMdSettings } from "react-icons/io";
import { MdCloudUpload, MdArrowBackIosNew } from "react-icons/md";
import { useRouter } from "next/router";
import Link from "next/link";
import { createRoom, useAuth } from "ceramic";

import useLocalWorld from "../../../helpers/localWorlds/useLocalWorld";
import SidebarLayout from "../../../layouts/SidebarLayout/SidebarLayout";
import { SceneSettingsDialog } from "../../../components/editor/SceneSettingsDialog";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const [openSettings, setOpenSettings] = useState(false);

  const { authenticated, connect } = useAuth();
  const world = useLocalWorld(id, openSettings);

  async function handlePublish() {
    if (!authenticated) {
      await connect();
    }

    const streamId = await createRoom(
      world.name,
      world.description,
      undefined,
      world.scene
    );
    router.push(`/room/${streamId}`);
  }

  if (!world) {
    return <div className="p-16">Scene not found.</div>;
  }

  return (
    <div className="p-16 space-y-4 max-w-6xl">
      <SceneSettingsDialog
        id={id}
        open={openSettings}
        setOpen={setOpenSettings}
      />

      <div className="flex items-center justify-between">
        <Link href="/editor" passHref>
          <div className="w-1/3 text-xl hover:cursor-pointer p-2 rounded-full">
            <MdArrowBackIosNew />
          </div>
        </Link>

        <div className="w-1/3 text-3xl flex justify-center">
          {world?.name ?? id}
        </div>

        <div className="w-1/3 flex justify-end">
          <div className="flex items-center h-min bg-neutral-200 rounded max-w-min">
            <Link href={`/editor/${id}/edit`} passHref>
              <div
                className="py-1.5 px-4 rounded-l hover:shadow hover:cursor-pointer
                       hover:bg-amber-300 text-xl"
              >
                <FaHammer />
              </div>
            </Link>
            <div
              onClick={() => setOpenSettings(true)}
              className="py-1.5 px-4 hover:shadow hover:cursor-pointer
                     hover:bg-amber-300 text-xl"
            >
              <IoMdSettings />
            </div>
            <div
              onClick={handlePublish}
              className="py-1.5 px-4 rounded-r hover:shadow hover:cursor-pointer
                     hover:bg-amber-300 text-xl"
            >
              <MdCloudUpload />
            </div>
          </div>
        </div>
      </div>

      {world?.image && (
        <div className="w-full ring-neutral-400 ring-1">
          <img src={world?.image} alt="scene preview" />
        </div>
      )}
    </div>
  );
}

Id.Layout = SidebarLayout;
