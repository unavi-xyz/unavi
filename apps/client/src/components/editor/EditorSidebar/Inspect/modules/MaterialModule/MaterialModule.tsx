import { useState } from "react";
import { AiOutlinePlus, AiFillFolderOpen } from "react-icons/ai";
import { IoMdClose } from "react-icons/io";

import { sceneManager, useStore } from "../../../../helpers/store";
import Module from "../Module";
import EditMaterial from "./EditMaterial";
import SelectMaterial from "./SelectMaterial";

export default function MaterialModule() {
  const selected = useStore((state) => state.selected);
  const properties = useStore(
    (state) => state.scene.instances[selected?.id]?.properties
  );
  const material = useStore((state) => {
    if (properties && "material" in properties) {
      return state.scene.materials[properties?.material];
    } else {
      return null;
    }
  });

  const [openSelect, setOpenSelect] = useState(false);

  if (!properties || !("material" in properties)) return null;

  function handleNewMaterial() {
    const newMaterial = sceneManager.newMaterial();
    sceneManager.editInstance(selected.id, { material: newMaterial });
  }

  function handleRemoveMaterial() {
    sceneManager.editInstance(selected.id, { material: undefined });
  }

  function handleSelectMaterial() {
    setOpenSelect(true);
  }

  return (
    <Module title="Material">
      <div className="space-y-4">
        <div className="relative">
          <div className="flex space-x-1">
            <div
              onClick={handleSelectMaterial}
              className="w-1/4 bg-neutral-50 hover:bg-neutral-100 rounded flex items-center justify-center"
            >
              <AiFillFolderOpen />
            </div>

            {material ? (
              <div
                className="w-full flex items-center justify-between space-x-2 bg-neutral-50 px-2 p-1 rounded
                         hover:cursor-default"
              >
                <div>{material.id}</div>
                <IoMdClose
                  onClick={handleRemoveMaterial}
                  className="text-neutral-500 hover:text-black"
                />
              </div>
            ) : (
              <div
                onClick={handleNewMaterial}
                className="w-full flex items-center space-x-2 bg-neutral-50 px-2 p-1 rounded
                       hover:bg-neutral-100 hover:cursor-default"
              >
                <AiOutlinePlus />
                <div>New Material</div>
              </div>
            )}
          </div>

          <SelectMaterial open={openSelect} setOpen={setOpenSelect} />
        </div>

        <EditMaterial id={properties.material} />
      </div>
    </Module>
  );
}
