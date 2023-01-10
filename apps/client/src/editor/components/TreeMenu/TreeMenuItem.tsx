import { useState } from "react";
import { HiOutlineCube } from "react-icons/hi";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { useEditorStore } from "../../store";

interface Props {
  id: string;
  depth?: number;
}

export default function TreeMenuItem({ id, depth = 0 }: Props) {
  const name = useNodeAttribute(id, "name");
  const childrenIds = useNodeAttribute(id, "children") ?? [];
  const selectedId = useEditorStore((state) => state.selectedId);

  const [open, setOpen] = useState(false);

  const isSelected = selectedId === id;
  const hasChildren = childrenIds.length > 0;

  return (
    <div>
      <div
        onClick={() => useEditorStore.setState({ selectedId: id })}
        style={{ paddingLeft: `${depth + 1}rem` }}
        className={`flex cursor-default select-none items-center space-x-1 text-sm text-neutral-800 ${
          isSelected ? "bg-sky-200 text-black" : "hover:bg-neutral-200"
        }`}
      >
        <div
          className={`w-3 shrink-0 hover:text-neutral-500 ${hasChildren && "cursor-pointer"}`}
          onClick={() => setOpen(!open)}
        >
          {hasChildren && (open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />)}
        </div>

        <HiOutlineCube className="shrink-0 text-lg" />

        <div className="overflow-x-hidden text-ellipsis">{name}</div>
      </div>

      <div>
        {open &&
          childrenIds.map((childId) => (
            <TreeMenuItem key={childId} id={childId} depth={depth + 1} />
          ))}
      </div>
    </div>
  );
}
