import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { MdCloudUpload, MdArrowBackIosNew } from "react-icons/md";
import { useRouter } from "next/router";
import Link from "next/link";
import { createRoom, useAuth } from "ceramic";

import { useLocalWorld } from "../../../helpers/localWorlds/useLocalWorld";
import { SceneSettingsDialog } from "../../../components/editor/SceneSettingsDialog";
import SidebarLayout from "../../../layouts/SidebarLayout/SidebarLayout";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const [openSettings, setOpenSettings] = useState(false);

  const { authenticated, connect } = useAuth();
  const world = useLocalWorld(id);

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
    <div className="space-y-4 h-2/3 flex flex-col">
      <div className="bg-white rounded-3xl shadow p-8 flex items-center justify-between">
        <div className="flex items-center space-x-8">
          <Link href="/editor" passHref>
            <div className="text-xl hover:cursor-pointer p-2 rounded-full">
              <MdArrowBackIosNew />
            </div>
          </Link>

          <div className="text-2xl">{world?.name ?? id}</div>
        </div>

        <div className="flex items-center space-x-4">
          <div
            onClick={handlePublish}
            className="h-12 w-12 hover:cursor-pointer flex items-center
          justify-center hover:bg-neutral-200 text-2xl rounded-xl"
          >
            <MdCloudUpload />
          </div>

          <div
            onClick={() => setOpenSettings(true)}
            className="h-12 w-12 hover:cursor-pointer flex items-center mr-4
          justify-center hover:bg-neutral-200 text-2xl rounded-xl"
          >
            <IoMdSettings />
          </div>

          <SceneSettingsDialog
            id={id}
            open={openSettings}
            setOpen={setOpenSettings}
          />
        </div>
      </div>

      <div className="h-full w-full flex space-x-4">
        <div className="w-3/5 h-full bg-white rounded-3xl shadow">
          {world?.image && (
            <img
              src={world?.image}
              alt="scene preview"
              className="w-full h-full rounded-3xl"
            />
          )}
        </div>

        <div className="w-2/5 flex flex-col space-y-4">
          <div className="h-full bg-white rounded-3xl shadow p-8 flex flex-col justify-between">
            <div className="space-y-4">
              <div className="text-2xl flex justify-center">About</div>
              <div className="text-xl">{world?.description}</div>
            </div>

            <div className="h-16">
              <Link href={`/editor/${id}/edit`} passHref>
                <div
                  className="h-full text-md rounded-full bg-black text-white shadow
                           hover:cursor-pointer hover:bg-opacity-80 hover:shadow-md transition-all
                           flex items-center justify-center"
                >
                  Edit Scene
                </div>
              </Link>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

Id.Layout = SidebarLayout;
