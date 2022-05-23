import { useAtomValue } from "jotai";
import { nanoid } from "nanoid";
import { useEffect, useRef, useState } from "react";
import { IoMdTrash } from "react-icons/io";
import { MdClose, MdOutlineAdd, MdOutlineFolderOpen } from "react-icons/md";

import { IMeshModule, Material } from "@wired-xr/scene";

import { selectedAtom } from "../../../../helpers/studio/atoms";
import { useStudioStore } from "../../../../helpers/studio/store";
import { round } from "../../../../helpers/utils/round";
import ColorInput from "../../../base/ColorInput";
import DropdownMenu from "../../../base/DropdownMenu";
import NumberInput from "../NumberInput";

export default function MaterialMenu() {
  const colorRef = useRef<HTMLInputElement>(null);
  const emissiveRef = useRef<HTMLInputElement>(null);

  const selected = useAtomValue(selectedAtom);

  const meshModule = selected?.modules.find((item) => item.type === "Mesh") as
    | IMeshModule
    | undefined;

  const materialId = meshModule?.materialId ?? "";
  const material = useStudioStore((state) => state.scene.materials[materialId]);
  const materials = useStudioStore((state) => state.scene.materials);

  const [open, setOpen] = useState(false);

  useEffect(() => {
    const interval = setInterval(() => {
      if (!material) return;

      useStudioStore.getState().updateMaterial(material.id, {
        color: colorRef.current?.value,
        emissive: emissiveRef.current?.value,
      });
    }, 25);

    if (material && colorRef.current) {
      colorRef.current.value = material.color;
    }
    if (material && emissiveRef.current) {
      emissiveRef.current.value = material.emissive;
    }

    return () => clearInterval(interval);
  }, [material]);

  if (!meshModule) return null;

  function handleNewMaterial() {
    if (!selected) return;

    const id = nanoid();
    const newMaterial: Material = {
      id,
      name: `New Material ${id.slice(0, 4)}`,
      color: "#ffffff",
      emissive: "#000000",
      opacity: 1,
      roughness: 1,
      metalness: 0,
      flatShading: false,
    };

    useStudioStore.getState().addMaterial(newMaterial);
    useStudioStore.getState().setMaterial(selected.id, newMaterial.id);
  }

  function handleRemoveMaterial() {
    if (!selected) return;
    useStudioStore.getState().setMaterial(selected.id, undefined);
  }

  function handleOpacityChange(value: string) {
    if (isNaN(Number(value)) || value === "" || !selected) return;
    const rounded = Math.min(Math.max(round(Number(value)), 0), 1);
    useStudioStore.getState().updateMaterial(material.id, { opacity: rounded });
  }

  function handleRoughnessChange(value: string) {
    if (isNaN(Number(value)) || value === "" || !selected) return;
    const rounded = Math.min(Math.max(round(Number(value)), 0), 1);
    useStudioStore
      .getState()
      .updateMaterial(material.id, { roughness: rounded });
  }

  function handleMetalnessChange(value: string) {
    if (isNaN(Number(value)) || value === "" || !selected) return;
    const rounded = Math.min(Math.max(round(Number(value)), 0), 1);
    useStudioStore
      .getState()
      .updateMaterial(material.id, { metalness: rounded });
  }

  function handleFlatShadingChange(value: boolean) {
    if (!selected) return;
    useStudioStore
      .getState()
      .updateMaterial(material.id, { flatShading: value });
  }

  return (
    <div className="space-y-1">
      <div className="flex space-x-1 py-1">
        <button
          onClick={(e) => {
            e.stopPropagation();
            setOpen((prev) => !prev);
          }}
          className="w-12 border rounded flex items-center justify-center transition
                   hover:bg-surfaceVariant cursor-default"
        >
          <MdOutlineFolderOpen />
        </button>

        {material ? (
          <div className="w-full flex relative">
            <input
              type="text"
              spellCheck={false}
              value={material.name}
              onChange={(e) => {
                useStudioStore
                  .getState()
                  .updateMaterial(materialId, { name: e.target.value });
              }}
              className="w-full flex items-center justify-center border rounded space-x-1 outline-none px-3"
            />
            <div className="absolute right-2 h-full">
              <button
                onClick={handleRemoveMaterial}
                className="h-full cursor-default text-outline hover:text-inherit transition p-1"
              >
                <MdClose className="h-full" />
              </button>
            </div>
          </div>
        ) : (
          <button
            onClick={handleNewMaterial}
            className="w-full flex items-center justify-center border rounded space-x-1
                       cursor-default transition hover:bg-surfaceVariant"
          >
            <MdOutlineAdd />
            <div>New Material</div>
          </button>
        )}
      </div>

      <DropdownMenu open={open} onClose={() => setOpen(false)}>
        <div className="p-2 space-y-1">
          {Object.values(materials).length === 0 ? (
            <div className="px-3 text-outline">No Materials</div>
          ) : (
            Object.entries(materials).map(([key, value]) => {
              return (
                <button
                  key={key}
                  onClick={() => {
                    if (!selected) return;
                    useStudioStore.getState().setMaterial(selected.id, key);
                    setOpen(false);
                  }}
                  className="group w-full rounded hover:bg-primaryContainer transition px-3
                             cursor-default flex items-center justify-between"
                >
                  {value.name}
                  <IoMdTrash
                    onClick={(e) => {
                      e.stopPropagation();
                      useStudioStore.getState().removeMaterial(key);
                      if (materialId === key && selected) {
                        useStudioStore
                          .getState()
                          .setMaterial(selected.id, undefined);
                      }
                    }}
                    className="opacity-0 group-hover:opacity-100 transition text-outline
                               hover:text-inherit"
                  />
                </button>
              );
            })
          )}
        </div>
      </DropdownMenu>

      {material && (
        <div key={material.id} className="space-y-1">
          <div className="flex">
            <div className="w-28">Color</div>
            <ColorInput inputRef={colorRef} />
          </div>

          <div className="flex">
            <div className="w-28">Emissive</div>
            <ColorInput inputRef={emissiveRef} />
          </div>

          <div className="flex">
            <div className="w-28">Opacity</div>
            <div>
              <NumberInput
                updatedValue={String(round(material.opacity))}
                onChange={(e) => handleOpacityChange(e.currentTarget.value)}
                max={1}
                min={0}
              />
            </div>
          </div>
          <div className="flex">
            <div className="w-28">Roughness</div>
            <div>
              <NumberInput
                updatedValue={String(round(material.roughness))}
                onChange={(e) => handleRoughnessChange(e.currentTarget.value)}
                max={1}
                min={0}
              />
            </div>
          </div>

          <div className="flex">
            <div className="w-28">Metalness</div>
            <div>
              <NumberInput
                updatedValue={String(round(material.metalness))}
                onChange={(e) => handleMetalnessChange(e.currentTarget.value)}
                max={1}
                min={0}
              />
            </div>
          </div>

          <div className="flex">
            <div className="w-28">Flat Shading</div>
            <div>
              <input
                type="checkbox"
                checked={material.flatShading}
                onChange={(e) =>
                  handleFlatShadingChange(e.currentTarget.checked)
                }
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
