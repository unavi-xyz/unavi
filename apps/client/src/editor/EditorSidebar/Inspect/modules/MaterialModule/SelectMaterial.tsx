import { Dispatch, MouseEvent, SetStateAction, useRef } from "react";
import { IoMdTrash } from "react-icons/io";

import { sceneManager, useStore } from "../../../../helpers/store";
import { useOutsideClick } from "./useOutsideClick";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export default function SelectMaterial({ open, setOpen }: Props) {
  const menuRef = useRef();

  const materials = useStore((state) => state.scene.materials);

  useOutsideClick(menuRef, () => setOpen(false));

  if (!open) return null;

  return (
    <div
      ref={menuRef}
      className="absolute top-100 left-0 z-10 w-full h-64 bg-neutral-200 rounded-b"
    >
      <div className="p-2">
        {Object.keys(materials).map((id) => {
          return <MaterialButton key={id} id={id} setOpen={setOpen} />;
        })}
      </div>
    </div>
  );
}

function MaterialButton({
  id,
  setOpen,
}: {
  id: string;
  setOpen: Dispatch<SetStateAction<boolean>>;
}) {
  function handleClick() {
    const selectedId = useStore.getState().selected.id;
    sceneManager.editInstance(selectedId, { material: id });
    setOpen(false);
  }

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    sceneManager.deleteMaterial(id);
  }

  return (
    <div
      onClick={handleClick}
      className="group flex items-center justify-between  px-2 rounded hover:bg-neutral-300 hover:cursor-default"
    >
      <div>{id}</div>
      <div onClick={handleDelete}>
        <IoMdTrash className="invisible group-hover:visible text-neutral-500 hover:text-black" />
      </div>
    </div>
  );
}
