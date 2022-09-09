import { nanoid } from "nanoid";
import { useState } from "react";
import { MdAdd, MdClose, MdOutlineFolderOpen } from "react-icons/md";

import { Material, WithMaterial } from "@wired-labs/engine";

import DropdownMenu from "../../../ui/base/DropdownMenu";
import { addMaterial } from "../../actions/AddMaterialAction";
import { setMaterial } from "../../actions/SetMaterial";
import { useEditorStore } from "../../store";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  entityId: string;
}

export default function MaterialComponent({ entityId }: Props) {
  const materialId = useEditorStore((state) => {
    const withMaterial = state.scene.entities[entityId] as WithMaterial;
    return withMaterial.material;
  });
  const materials = useEditorStore((state) => state.scene.materials);
  const material = materialId ? materials[materialId] : null;

  const [open, setOpen] = useState(false);

  function createMaterial() {
    const id = nanoid();
    const material: Material = {
      id,
      name: "New Material",
      color: [1, 0, 1],
      roughness: 0.5,
      metalness: 0.5,
    };

    addMaterial(material);
    setMaterial(entityId, material.id);
  }

  return (
    <ComponentMenu title="Material">
      <div className="w-full h-7 flex justify-between space-x-2">
        <button
          onClick={() => setOpen((prev) => !prev)}
          className={`h-full hover:bg-primaryContainer transition flex justify-center
                      items-center rounded-md cursor-default w-1/3 min-w-fit space-y-1`}
        >
          <MdOutlineFolderOpen />
        </button>

        <div className="w-full">
          {material ? (
            <div className="w-full h-full flex items-center justify-between bg-neutral-100 shadow-inner rounded-md">
              <div className="pl-4">{material.name}</div>

              {material && (
                <button
                  onClick={() => setMaterial(entityId, undefined)}
                  className={`h-full flex items-center px-2 text-lg transition
                          text-outline hover:text-black cursor-default`}
                >
                  <MdClose />
                </button>
              )}
            </div>
          ) : (
            <button
              onClick={createMaterial}
              className={`w-full h-full bg-neutral-100 rounded-md hover:bg-primaryContainer transition
                        flex justify-center items-center space-x-1 cursor-default shadow-inner`}
            >
              <MdAdd className="text-lg" />
              <div>New Material</div>
            </button>
          )}

          <DropdownMenu open={open} onClose={() => setOpen(false)}>
            <div className="p-2 space-y-1 flex flex-col overflow-y-auto max-h-80">
              {Object.values(materials).length > 0
                ? Object.values(materials).map((material) => {
                    if (material.id === materialId) return null;

                    return (
                      <button
                        key={material.id}
                        onClick={() => setMaterial(entityId, material.id)}
                        className="w-full cursor-default hover:bg-primaryContainer rounded-md transition"
                      >
                        {material.name}
                      </button>
                    );
                  })
                : "No materials"}
            </div>
          </DropdownMenu>
        </div>
      </div>
      <div className="pt-2" />
      {material && (
        <MenuRows titles={["Color"]}>
          <div>Colorrr</div>
        </MenuRows>
      )}{" "}
    </ComponentMenu>
  );
}
