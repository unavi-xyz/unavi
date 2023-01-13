import { Image, Material, Quad, Texture } from "engine";
import { useState } from "react";
import { BiImageAdd } from "react-icons/bi";
import { MdAdd, MdClose, MdDelete, MdOutlineFolderOpen } from "react-icons/md";

import ButtonFileInput from "../../../ui/ButtonFileInput";
import DropdownMenu from "../../../ui/DropdownMenu";
import { hexToRgb } from "../../../utils/rgb";
import { addMaterial } from "../../actions/AddMaterialAction";
import { removeMaterial } from "../../actions/RemoveMaterialAction";
import { updateMaterial } from "../../actions/UpdateMaterialAction";
import { updateMesh } from "../../actions/UpdateMeshAction";
import { useMaterial } from "../../hooks/useMaterial";
import { useMesh } from "../../hooks/useMesh";
import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import ColorInput from "../ui/ColorInput";
import NumberInput from "../ui/NumberInput";
import TextInput from "../ui/TextInput";
import MenuRows from "./ui/MenuRows";

interface Props {
  nodeId: string;
}

export default function MaterialComponent({ nodeId }: Props) {
  const node = useNodeAttribute(nodeId);
  const meshId = useSubscribeValue(node?.meshId$);
  const mesh = useMesh(meshId);
  const materialId = useSubscribeValue(mesh?.materialId$);
  const material = useMaterial(materialId);

  const name = useSubscribeValue(material?.name$);
  const color = useSubscribeValue(material?.color$);
  const colorTexture = useSubscribeValue(material?.colorTexture$);
  const roughness = useSubscribeValue(material?.roughness$);
  const metalness = useSubscribeValue(material?.metalness$);
  const alpha = useSubscribeValue(material?.alpha$);

  const materials$ = useEditorStore((state) => state.engine?.scene.materials$);
  const _materials = useSubscribeValue(materials$);
  const materials = Object.values(_materials ?? {}).filter((m) => !m.isInternal);

  const [open, setOpen] = useState(false);

  function createMaterial() {
    if (!mesh) throw new Error("Mesh not found");
    const newMaterial = new Material();
    addMaterial(newMaterial);
    updateMesh(mesh.id, { materialId: newMaterial.id });
  }

  if (!mesh) return null;

  return (
    <>
      <div className="pt-10 text-xl font-bold">
        <div>Material</div>
      </div>

      <div className="flex h-7 w-full justify-between space-x-4">
        <button
          onClick={() => setOpen(true)}
          className={`flex h-full w-1/3 min-w-fit cursor-default
                      items-center justify-center space-y-1 rounded-md transition hover:bg-sky-100`}
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
                onClick={() => updateMesh(mesh.id, { materialId: null })}
                className="flex h-full cursor-default items-center px-2 text-lg text-neutral-100 transition hover:text-black"
              >
                <MdClose />
              </button>
            </div>
          ) : (
            <button
              onClick={createMaterial}
              className={`flex h-full w-full cursor-default items-center justify-center space-x-1 rounded-md bg-neutral-100 shadow-inner transition hover:bg-sky-100`}
            >
              <MdAdd className="text-lg" />
              <div>New Material</div>
            </button>
          )}

          <DropdownMenu open={open} onClose={() => setOpen(false)} fullWidth>
            <div className="flex max-h-96 flex-col space-y-1 overflow-x-hidden p-2">
              {materials.length > 0 ? (
                materials.map((material) => {
                  return (
                    <div key={material.id}>
                      <DropdownMaterialButton
                        materialId={material.id}
                        selectedId={materialId}
                        onClick={() => updateMesh(mesh.id, { materialId: material.id })}
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
        <MenuRows titles={["Color", "Roughness", "Metalness", "Opacity"]}>
          <div className="flex h-6 space-x-2">
            <div className="w-full">
              <ColorInput
                rgbValue={[color[0], color[1], color[2]]}
                onChange={(e) => {
                  const value = e.target.value;

                  const rgb = hexToRgb(value);
                  const normalized: Quad = [rgb[0] / 255, rgb[1] / 255, rgb[2] / 255, 255];

                  updateMaterial(materialId, { color: normalized });
                }}
              />
            </div>
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

          <NumberInput
            value={alpha ?? 1}
            step={0.01}
            max={1}
            min={0}
            onChange={(e) => {
              const value = e.target.value;

              const num = parseFloat(value);
              const rounded = Math.round(num * 1000) / 1000;

              const alphaMode = rounded === 1 ? "OPAQUE" : "BLEND";

              updateMaterial(materialId, {
                alpha: rounded,
                alphaMode,
              });
            }}
          />
        </MenuRows>
      )}

      {materialId && (
        <MenuRows titles={["Color Texture"]}>
          <div className="h-6">
            {colorTexture ? (
              <div className="flex h-full w-full items-center justify-between rounded-md bg-neutral-100 shadow-inner">
                <TextInput
                  value={colorTexture.name}
                  onChange={(e) => {
                    const json = colorTexture.toJSON();

                    updateMaterial(materialId, {
                      colorTexture: {
                        ...json,
                        name: e.target.value,
                      },
                    });
                  }}
                />

                <button
                  onClick={() => {
                    updateMaterial(materialId, { colorTexture: null });

                    const { engine } = useEditorStore.getState();
                    if (!engine) throw new Error("Engine not found");

                    if (colorTexture.imageId) engine.scene.removeImage(colorTexture.imageId);
                  }}
                  className="flex h-full cursor-default items-center px-2 text-lg text-neutral-100 transition hover:text-black"
                >
                  <MdClose />
                </button>
              </div>
            ) : (
              <ButtonFileInput
                rounded="small"
                accept="image/*"
                onChange={async (e) => {
                  const { engine } = useEditorStore.getState();
                  if (!engine) return;

                  const file = e.target.files?.[0];
                  if (!file) return;

                  const buffer = await file.arrayBuffer();
                  const array = new Uint8Array(buffer);

                  const bitmap = await createImageBitmap(new Blob([array]));

                  const image = new Image({
                    array,
                    mimeType: file.type,
                    bitmap,
                  });

                  engine.scene.loadJSON({
                    images: [image.toJSON()],
                  });

                  const texture = new Texture();
                  texture.imageId = image.id;
                  texture.name = file.name;

                  updateMaterial(materialId, {
                    colorTexture: texture.toJSON(),
                  });
                }}
              >
                <BiImageAdd />
              </ButtonFileInput>
            )}
          </div>
        </MenuRows>
      )}
    </>
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
  const [hovering, setHovering] = useState(false);

  const material = useMaterial(materialId);
  if (!material) return null;

  const selectedClass = materialId === selectedId ? "bg-sky-100" : "";
  const hoveringClass = hovering ? "opacity-100" : "";

  return (
    <div className="relative">
      <button
        onClick={onClick}
        onMouseOver={() => setHovering(true)}
        onMouseLeave={() => setHovering(false)}
        className={`group flex w-full cursor-default items-center justify-between rounded-md pl-4 pr-2 transition hover:bg-sky-100 ${selectedClass}`}
      >
        <div className="overflow-hidden text-ellipsis whitespace-nowrap">
          {material.name || materialId}
        </div>

        <div
          onPointerUp={(e) => {
            e.stopPropagation();
            e.preventDefault();
            removeMaterial(materialId);
          }}
          className={`text-neutral-500 opacity-0 transition hover:text-black ${hoveringClass}`}
        >
          <MdDelete />
        </div>
      </button>
    </div>
  );
}
