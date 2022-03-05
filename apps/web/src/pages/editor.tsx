import { useState } from "react";
import { AiOutlinePlus, AiOutlineUpload } from "react-icons/ai";

import { NewWorldDialog } from "../components/editor/NewWorldDialog";
import SidebarLayout from "../layouts/SidebarLayout/SidebarLayout";

export default function Editor() {
  const [openNew, setOpenNew] = useState(false);

  function handleNew() {
    setOpenNew(true);
  }

  function handleImport() {}

  return (
    <div className="p-16 space-y-4">
      <NewWorldDialog open={openNew} setOpen={setOpenNew} />

      <div className="flex items-center space-x-8">
        <div className="text-3xl">Editor</div>

        <div className="flex bg-neutral-200 rounded">
          <div
            onClick={handleNew}
            className="py-1.5 px-4 rounded-l hover:shadow hover:cursor-pointer
                       hover:bg-amber-300 text-xl"
          >
            <AiOutlinePlus />
          </div>
          <div
            onClick={handleImport}
            className="py-1.5 px-4 rounded-r hover:shadow hover:cursor-pointer
                       hover:bg-amber-300 text-xl"
          >
            <AiOutlineUpload />
          </div>
        </div>
      </div>
      <div>
        It looks like you don{"'"}t have any worlds.{" "}
        <span
          onClick={handleNew}
          className="text-amber-600 underline hover:decoration-2 hover:cursor-pointer"
        >
          Click here
        </span>{" "}
        to get started.
      </div>
    </div>
  );
}

Editor.Layout = SidebarLayout;
