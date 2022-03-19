import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { MdCloudUpload, MdArrowBackIosNew } from "react-icons/md";
import { useRouter } from "next/router";
import Link from "next/link";
import { createSpace, uploadFileToIpfs, useAuth } from "ceramic";

import { dataUrlToFile } from "../../../helpers/files";
import { useLocalScene } from "../../../components/scene/localScenes/useLocalScene";

import { IconButton } from "../../../components/base";
import SceneSettingsDialog from "../../../components/scene/SceneSettingsDialog";
import SidebarLayout from "../../../layouts/SidebarLayout/SidebarLayout";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const { authenticated, connect } = useAuth();
  const localScene = useLocalScene(id);

  const [openSettings, setOpenSettings] = useState(false);

  async function handlePublish() {
    if (!authenticated) await connect();

    //upload textures
    const sceneCopy = { ...localScene.scene };
    Promise.all(
      Object.values(sceneCopy.textures).map(async ({ value, name }) => {
        const file = await dataUrlToFile(value, name);
        await uploadFileToIpfs(file);
      })
    );

    //remove textures from scene
    Object.keys(sceneCopy.textures).forEach((key) => {
      sceneCopy.textures[key] = null;
    });

    //upload models
    Promise.all(
      Object.values(sceneCopy.models).map(async ({ value, name }) => {
        const file = await dataUrlToFile(value, name);
        await uploadFileToIpfs(file);
      })
    );

    //remove models from scene
    Object.keys(sceneCopy.models).forEach((key) => {
      sceneCopy.models[key] = null;
    });

    //upload preview image
    const image = localScene?.image
      ? await dataUrlToFile(localScene.image, "preview")
      : undefined;

    //create the space
    const streamId = await createSpace(
      localScene.name,
      localScene.description,
      image,
      sceneCopy
    );

    router.push(`/space/${streamId}`);
  }

  if (!localScene) {
    return <div className="p-16">Scene not found.</div>;
  }

  return (
    <div className="flex flex-col space-y-4 h-full">
      <div className="card flex items-center justify-between">
        <div className="flex items-center space-x-8 h-8">
          <Link href="/editor" passHref>
            <div className="text-xl hover:cursor-pointer p-2 rounded-full">
              <MdArrowBackIosNew />
            </div>
          </Link>

          <div className="text-2xl">{localScene?.name ?? id}</div>
        </div>

        <div className="flex items-center space-x-4 h-8">
          <IconButton onClick={handlePublish}>
            <MdCloudUpload />
          </IconButton>

          <IconButton onClick={() => setOpenSettings(true)}>
            <IoMdSettings />
          </IconButton>

          <SceneSettingsDialog
            id={id}
            open={openSettings}
            setOpen={setOpenSettings}
          />
        </div>
      </div>

      <div className="h-full md:h-1/2 md:grid md:grid-cols-3 md:gap-4 space-y-4 md:space-y-0">
        <div className="w-full col-span-2 h-64 md:h-full max-h-[800px] card-borderless">
          {localScene?.image && (
            <img
              src={localScene.image}
              alt="scene image"
              className="w-full h-full rounded-3xl object-cover"
            />
          )}
        </div>

        <div className="card flex flex-col space-y-4">
          <div className="text-2xl font-medium flex justify-center">About</div>

          <div className="relative overflow-auto h-36 md:h-full">
            <div className="absolute top-0 left-0 px-2 whitespace-pre-wrap">
              {localScene?.description}
            </div>
          </div>

          <div className="h-16">
            <Link href={`/editor/${id}/edit`} passHref>
              <div
                className="h-full text-md rounded-full bg-black text-white justify-center
                           hover:cursor-pointer transition-all flex items-center"
              >
                Edit Scene
              </div>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}

Id.Layout = SidebarLayout;
