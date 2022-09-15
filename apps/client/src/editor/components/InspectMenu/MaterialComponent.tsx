import { Material, WithMaterial } from "@wired-labs/engine";
import { nanoid } from "nanoid";
import { useState } from "react";
import { MdAdd, MdClose, MdDelete, MdOutlineFolderOpen } from "react-icons/md";

import DropdownMenu from "../../../ui/base/DropdownMenu";
import { hexToRgb } from "../../../utils/rgb";
import { addMaterial } from "../../actions/AddMaterialAction";
import { editMaterial } from "../../actions/EditMaterialAction";
import { removeMaterial } from "../../actions/RemoveMaterialAction";
import { setMaterial } from "../../actions/SetMaterial";
import { useEditorStore } from "../../store";
import ColorInput from "../ui/ColorInput";
import NumberInput from "../ui/NumberInput";
import TextInput from "../ui/TextInput";
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
  const name = useEditorStore((state) => {
    const withMaterial = state.scene.entities[entityId] as WithMaterial;
    if (!withMaterial.material) return null;
    const material = state.scene.materials[withMaterial.material];
    return material.name;
  });
  const color = useEditorStore((state) => {
    const withMaterial = state.scene.entities[entityId] as WithMaterial;
    if (!withMaterial.material) return null;
    const material = state.scene.materials[withMaterial.material];
    return material.color;
  });
  const roughness = useEditorStore((state) => {
    const withMaterial = state.scene.entities[entityId] as WithMaterial;
    if (!withMaterial.material) return null;
    const material = state.scene.materials[withMaterial.material];
    return material.roughness;
  });
  const metalness = useEditorStore((state) => {
    const withMaterial = state.scene.entities[entityId] as WithMaterial;
    if (!withMaterial.material) return null;
    const material = state.scene.materials[withMaterial.material];
    return material.metalness;
  });
  const materials = useEditorStore((state) => state.scene.materials);

  const [open, setOpen] = useState(false);

  function createMaterial() {
    const id = nanoid();
    const material: Material = {
      id,
      name: "New Material",
      color: [1, 1, 1],
      roughness: 1,
      metalness: 0,
    };

    addMaterial(material);
    setMaterial(entityId, material.id);
  }

  return (
    <ComponentMenu title="Material">
      <div className="flex h-7 w-full justify-between space-x-4">
        <button
          onClick={(e) => setOpen(true)}
          className={`hover:bg-primaryContainer flex h-full w-1/3 min-w-fit
                      cursor-default items-center justify-center space-y-1 rounded-md transition`}
        >
          <MdOutlineFolderOpen />
        </button>

        <div className="w-full">
          {materialId ? (
            <div className="flex h-full w-full items-center justify-between rounded-md bg-neutral-100 shadow-inner">
              <TextInput
                value={name ?? ""}
                onChange={(e) => {
                  const { scene } = useEditorStore.getState();
                  const material = scene.materials[materialId];
                  material.name = e.target.value;
                  editMaterial(material);
                }}
              />

              <button
                onClick={() => setMaterial(entityId, undefined)}
                className={`text-outline flex h-full cursor-default items-center px-2
                              text-lg transition hover:text-black`}
              >
                <MdClose />
              </button>
            </div>
          ) : (
            <button
              onClick={createMaterial}
              className={`hover:bg-primaryContainer flex h-full w-full cursor-default items-center
                          justify-center space-x-1 rounded-md bg-neutral-100 shadow-inner transition`}
            >
              <MdAdd className="text-lg" />
              <div>New Material</div>
            </button>
          )}

          <DropdownMenu open={open} onClose={() => setOpen(false)}>
            <div className="flex max-h-52 flex-col space-y-1 overflow-y-auto p-2">
              {Object.values(materials).length > 0 ? (
                Object.values(materials).map((material) => {
                  return (
                    <div key={material.id}>
                      <DropdownMaterialButton
                        materialId={material.id}
                        selectedId={materialId}
                        onClick={() => setMaterial(entityId, material.id)}
                      />
                    </div>
                  );
                })
              ) : (
                <div className="flex justify-center">No materials</div>
              )}
            </div>
          </DropdownMenu>
        </div>
      </div>

      {materialId && color && (
        <MenuRows titles={["Color", "Roughness", "Metalness"]}>
          <div className="h-6">
            <ColorInput
              rgbValue={color}
              onChange={(e) => {
                const value = e.target.value;
                const rgb = hexToRgb(value);
                const normalized: [number, number, number] = [
                  rgb[0] / 255,
                  rgb[1] / 255,
                  rgb[2] / 255,
                ];

                const { scene } = useEditorStore.getState();
                const material = scene.materials[materialId];

                material.color = normalized;
                editMaterial(material);
              }}
            />
          </div>

          <NumberInput
            value={roughness ?? 0}
            step={0.01}
            max={1}
            min={0}
            onChange={(e) => {
              const num = parseFloat(e.target.value);
              const rounded = Math.round(num * 1000) / 1000;

              const { scene } = useEditorStore.getState();
              const material = scene.materials[materialId];

              material.roughness = rounded;
              editMaterial(material);
            }}
          />

          <NumberInput
            value={metalness ?? 0}
            step={0.01}
            max={1}
            min={0}
            onChange={(e) => {
              const num = parseFloat(e.target.value);
              const rounded = Math.round(num * 1000) / 1000;

              const { scene } = useEditorStore.getState();
              const material = scene.materials[materialId];

              material.metalness = rounded;
              editMaterial(material);
            }}
          />
        </MenuRows>
      )}
    </ComponentMenu>
  );
}

function DropdownMaterialButton({
  materialId,
  selectedId,
  onClick,
}: {
  materialId: string;
  selectedId: string | undefined;
  onClick: () => void;
}) {
  const material = useEditorStore((state) => state.scene.materials[materialId]);
  if (!material) return null;

  const selectedClass = materialId === selectedId ? "bg-primaryContainer" : "";

  return (
    <div className="group relative">
      <button
        onClick={onClick}
        className={`hover:bg-primaryContainer w-full cursor-default
                    rounded-md transition ${selectedClass}`}
      >
        <div>{material.name || materialId}</div>
      </button>

      <div
        onPointerUp={(e) => {
          e.stopPropagation();
          e.preventDefault();
          removeMaterial(materialId);
        }}
        className={`text-outline absolute right-2 top-0 z-10 flex h-full items-center
                    opacity-0 transition hover:text-black group-hover:opacity-100`}
      >
        <MdDelete />
      </div>
    </div>
  );
}
