import { Material, Triplet } from "@wired-labs/engine";
import { useState } from "react";
import { MdAdd, MdClose, MdDelete, MdOutlineFolderOpen } from "react-icons/md";

import DropdownMenu from "../../../ui/base/DropdownMenu";
import { hexToRgb } from "../../../utils/rgb";
import { addMaterial } from "../../actions/AddMaterialAction";
import { removeMaterial } from "../../actions/RemoveMaterialAction";
import { updateEntity } from "../../actions/UpdateEntityAction";
import { updateMaterial } from "../../actions/UpdateMaterialAction";
import { useEntity } from "../../hooks/useEntity";
import { useMaterial } from "../../hooks/useMaterial";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
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
  const entity = useEntity(entityId);
  const materialId = useSubscribeValue(entity?.materialId$);

  const material = useMaterial(materialId);
  const name = useSubscribeValue(material?.name$);
  const color = useSubscribeValue(material?.color$);
  const roughness = useSubscribeValue(material?.roughness$);
  const metalness = useSubscribeValue(material?.metalness$);

  const materials$ = useEditorStore((state) => state.engine?.scene.materials$);
  const materials = useSubscribeValue(materials$);

  const [open, setOpen] = useState(false);

  function createMaterial() {
    const newMaterial = new Material();
    addMaterial(newMaterial);
    updateEntity(entityId, { materialId: newMaterial.id });
  }

  return (
    <ComponentMenu title="Material">
      <div className="flex h-7 w-full justify-between space-x-4">
        <button
          onClick={(e) => setOpen(true)}
          className={`flex h-full w-1/3 min-w-fit cursor-default
                      items-center justify-center space-y-1 rounded-md transition hover:bg-primaryContainer`}
        >
          <MdOutlineFolderOpen />
        </button>

        <div className="w-full">
          {materialId ? (
            <div className="flex h-full w-full items-center justify-between rounded-md bg-neutral-100 shadow-inner">
              <TextInput
                value={name ?? ""}
                onChange={(e) => {
                  updateMaterial(materialId, { name: e.target.value });
                }}
              />

              <button
                onClick={() => updateEntity(entityId, { materialId: null })}
                className="flex h-full cursor-default items-center px-2 text-lg text-outline transition hover:text-black"
              >
                <MdClose />
              </button>
            </div>
          ) : (
            <button
              onClick={createMaterial}
              className={`flex h-full w-full cursor-default items-center justify-center
                          space-x-1 rounded-md bg-neutral-100 shadow-inner transition hover:bg-primaryContainer`}
            >
              <MdAdd className="text-lg" />
              <div>New Material</div>
            </button>
          )}

          <DropdownMenu open={open} onClose={() => setOpen(false)}>
            <div className="flex max-h-52 flex-col space-y-1 overflow-y-auto p-2">
              {materials && Object.values(materials).length > 0 ? (
                Object.values(materials).map((material) => {
                  return (
                    <div key={material.id}>
                      <DropdownMaterialButton
                        materialId={material.id}
                        selectedId={materialId}
                        onClick={() =>
                          updateEntity(entityId, { materialId: material.id })
                        }
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
                const normalized: Triplet = [
                  rgb[0] / 255,
                  rgb[1] / 255,
                  rgb[2] / 255,
                ];

                updateMaterial(materialId, { color: normalized });
              }}
            />
          </div>

          <NumberInput
            value={roughness ?? 0}
            step={0.01}
            max={1}
            min={0}
            onChange={(e) => {
              const value = e.target.value;

              const num = parseFloat(value);
              const rounded = Math.round(num * 1000) / 1000;

              updateMaterial(materialId, { roughness: rounded });
            }}
          />

          <NumberInput
            value={metalness ?? 0}
            step={0.01}
            max={1}
            min={0}
            onChange={(e) => {
              const value = e.target.value;

              const num = parseFloat(value);
              const rounded = Math.round(num * 1000) / 1000;

              updateMaterial(materialId, { metalness: rounded });
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
  selectedId: string | null;
  onClick: () => void;
}) {
  const material = useMaterial(materialId);
  if (!material) return null;

  const selectedClass = materialId === selectedId ? "bg-primaryContainer" : "";

  return (
    <div className="group relative">
      <button
        onClick={onClick}
        className={`w-full cursor-default rounded-md
                    transition hover:bg-primaryContainer ${selectedClass}`}
      >
        <div>{material.name || materialId}</div>
      </button>

      <div
        onPointerUp={(e) => {
          e.stopPropagation();
          e.preventDefault();
          removeMaterial(materialId);
        }}
        className={`absolute right-2 top-0 z-10 flex h-full items-center text-outline
                    opacity-0 transition hover:text-black group-hover:opacity-100`}
      >
        <MdDelete />
      </div>
    </div>
  );
}
