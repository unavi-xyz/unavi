import { useState } from "react";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { TreeObject } from "scene";

import { useStudioStore } from "../../../helpers/studio/store";

interface Props {
  object: TreeObject;
}

export default function TreeMenuItem({ object }: Props) {
  const selectedId = useStudioStore((state) => state.selectedId);
  const [open, setOpen] = useState(true);

  const hasChildren = object.children.length > 0;
  const isSelected = selectedId === object.id;
  const selectedClass = isSelected ? "bg-neutral-100" : "";

  return (
    <div className="space-y-1">
      <div
        onClick={(e) => {
          e.stopPropagation();
          useStudioStore.setState({ selectedId: object.id });
        }}
        className={`font-bold hover:bg-neutral-100 rounded-md px-2 cursor-pointer
                   flex items-center ${selectedClass}`}
      >
        <div
          onClick={() => setOpen((prev) => !prev)}
          className="w-5 text-neutral-500 hover:text-neutral-400"
        >
          {hasChildren ? (
            open ? (
              <IoMdArrowDropdown />
            ) : (
              <IoMdArrowDropright />
            )
          ) : null}
        </div>

        <div>{object.name}</div>
      </div>

      <div className="pl-4 space-y-1">
        {open &&
          object.children.map((child) => (
            <TreeMenuItem key={child.id} object={child} />
          ))}
      </div>
    </div>
  );
}
