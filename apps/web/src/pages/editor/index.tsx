import Link from "next/link";
import { useEffect, useState } from "react";
import { AiOutlinePlus, AiOutlineUpload } from "react-icons/ai";
import LocalWorldCard from "../../components/editor/LocalWorldCard";
import { getLocalWorldIds } from "../../helpers/localWorlds/db";

import { NewWorldDialog } from "../../components/editor/NewWorldDialog";
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

  function handleImport() {}

  return (
    <div className="p-16 space-y-4">
      <NewWorldDialog open={openNew} setOpen={setOpenNew} />

      <div className="text-3xl">Editor</div>

      <div className="flex items-center space-x-6">
        <div className="text-xl">Local Worlds</div>

        <div className="flex h-min items-center bg-neutral-200 rounded">
          <div
            onClick={handleNew}
            className="py-1 px-3 rounded-l hover:shadow hover:cursor-pointer hover:bg-amber-300 text-xl"
          >
            <AiOutlinePlus />
          </div>
          <div
            onClick={handleImport}
            className="py-1 px-3 rounded-r hover:shadow hover:cursor-pointer hover:bg-amber-300 text-xl"
          >
            <AiOutlineUpload />
          </div>
        </div>
      </div>

      <div className="overflow-auto">
        {localWorlds &&
          (localWorlds.length > 0 ? (
            <div className="flex space-x-2">
              {localWorlds.map((id) => {
                return (
                  <Link key={id} href={`/editor/${id}`} passHref>
                    <div>
                      <LocalWorldCard id={id} />
                    </div>
                  </Link>
                );
              })}
            </div>
          ) : (
            <p className="text-neutral-600">
              It looks like you don{"'"}t have any worlds.{" "}
              <span
                onClick={handleNew}
                className="text-amber-600 underline hover:decoration-2 hover:cursor-pointer"
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
