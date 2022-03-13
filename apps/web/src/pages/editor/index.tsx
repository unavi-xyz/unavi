import { useEffect, useState } from "react";
import { AiOutlinePlus } from "react-icons/ai";
import Link from "next/link";

import { getLocalWorldIds } from "../../helpers/localWorlds/db";
import { NewSceneDialog } from "../../components/editor/NewSceneDialog";
import SceneCard from "../../components/editor/SceneCard";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";

export default function Editor() {
  const [openNew, setOpenNew] = useState(false);

  const [localWorlds, setLocalWorlds] = useState<string[]>();

  useEffect(() => {
    getLocalWorldIds().then((res) => {
      setLocalWorlds(res);
    });
  }, []);

  function handleNew() {
    setOpenNew(true);
  }

  return (
    <div className="space-y-4 h-full w-full flex flex-col">
      <NewSceneDialog open={openNew} setOpen={setOpenNew} />

      <div className="bg-white rounded-3xl shadow p-8 flex items-center justify-between">
        <div className="text-2xl">Scenes</div>

        <div className="flex items-center space-x-4">
          <div
            onClick={handleNew}
            className="h-12 w-12 hover:cursor-pointer flex items-center transition-all
                       justify-center hover:bg-neutral-200 text-2xl rounded-xl"
          >
            <AiOutlinePlus />
          </div>
        </div>
      </div>

      <div className="h-full overflow-auto bg-white rounded-3xl shadow p-8">
        {localWorlds &&
          (localWorlds.length > 0 ? (
            <div className="grid grid-flow-row xl:grid-cols-5 lg:grid-cols-4 md:grid-cols-3">
              {localWorlds.map((id) => {
                return (
                  <div key={id} className="p-2">
                    <Link href={`/editor/${id}`} passHref>
                      <div>
                        <SceneCard id={id} />
                      </div>
                    </Link>
                  </div>
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
          ))}
      </div>
    </div>
  );
}

Editor.Layout = SidebarLayout;
