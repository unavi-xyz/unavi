import { useState } from "react";
import { AiOutlinePlus } from "react-icons/ai";
import Link from "next/link";

import { useLocalSceneIds } from "../../helpers/localScenes/useLocalSceneIds";
import { IconButton } from "../../components/base";
import NewSceneDialog from "../../components/home/scene/NewSceneDialog";
import SceneCard from "../../components/home/scene/SceneCard";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";

export default function Editor() {
  const [openNew, setOpenNew] = useState(false);

  const localScenesIds = useLocalSceneIds();

  function handleNew() {
    setOpenNew(true);
  }

  return (
    <>
      <NewSceneDialog open={openNew} setOpen={setOpenNew} />

      <div className="space-y-4 w-full flex flex-col">
        <div className="card flex items-center justify-between ">
          <div className="text-2xl">Scenes</div>

          <div className="flex items-center space-x-4 h-8">
            <IconButton onClick={handleNew}>
              <AiOutlinePlus />
            </IconButton>
          </div>
        </div>

        <div className="h-full overflow-auto card">
          {localScenesIds?.length > 0 ? (
            <div className="grid grid-flow-row gap-8 lg:grid-cols-3 md:grid-cols-2">
              {localScenesIds.map((id) => {
                return (
                  <Link key={id} href={`/editor/${id}`} passHref>
                    <div className="h-80">
                      <SceneCard id={id} />
                    </div>
                  </Link>
                );
              })}
            </div>
          ) : (
            <p className="text-neutral-500 text-lg">
              It looks like you don{"'"}t have any scenes.{" "}
              <span
                onClick={handleNew}
                className="text-amber-500 underline hover:decoration-2 hover:cursor-pointer"
              >
                Click here
              </span>{" "}
              to get started.
            </p>
          )}
        </div>
      </div>
    </>
  );
}

Editor.Layout = SidebarLayout;
