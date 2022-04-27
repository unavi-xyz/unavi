import { ChangeEvent } from "react";
import { IoMdClose } from "react-icons/io";
import { Material, Material as Material3d } from "3d";

import { sceneManager, useStore } from "../../../../helpers/store";

import ImageInput from "../../inputs/ImageInput";
import ColorInput from "../../inputs/ColorInput";
import NumberField from "../../inputs/NumberField";
import CheckboxInput from "../../inputs/CheckboxInput";
import { Select } from "../../../../../components/base";

interface Props {
  id: string;
}

export default function EditMaterial({ id }: Props) {
  const assets = useStore((state) => state.scene.assets);
  const materials = useStore((state) => state.scene.materials);

  if (!id) return null;

  const material = materials[id];
  const texture = assets[material?.texture];

  if (!material) return null;

  function editMaterial(changes: Partial<Material>) {
    sceneManager.editMaterial(id, changes);
  }

  function getEditMaterial(key: keyof Material3d) {
    return (value: Material3d[typeof key]) => editMaterial({ [key]: value });
  }

  async function handleTextureChange(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files[0];
    e.target.value = null;

    const textureId = await sceneManager.newAsset(file);
    editMaterial({ texture: textureId });
  }

  function handleDeleteTexture() {
    sceneManager.deleteAsset(material.texture);
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center w-full">
        <div className="w-1/4">Type</div>

        <div className="w-1/2">
          <Select
            value={material.type}
            options={["physical", "toon"]}
            onChange={getEditMaterial("type")}
          />
        </div>
      </div>

      {material.type === "physical" && (
        <>
          <NumberField
            title="Reflectivity"
            value={material.reflectivity}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("reflectivity")}
          />

          <NumberField
            title="Roughness"
            value={material.roughness}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("roughness")}
          />

          <NumberField
            title="Metalness"
            value={material.metalness}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("metalness")}
          />

          <NumberField
            title="Clearcoat"
            value={material.clearcoat}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("clearcoat")}
          />

          <NumberField
            title="Sheen"
            value={material.sheen}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("sheen")}
          />

          <div className="flex items-center w-full">
            <div className="w-1/4">Sheen Color</div>

            <div className="w-1/4">
              <ColorInput
                id="sheenColor"
                value={material.sheenColor}
                onChange={getEditMaterial("sheenColor")}
              />
            </div>
          </div>

          <div className="flex items-center w-full">
            <div className="w-1/4">Flat Shading</div>

            <div className="w-1/4">
              <CheckboxInput
                checked={material.flatShading}
                onChange={getEditMaterial("flatShading")}
              />
            </div>
          </div>
        </>
      )}

      <div className="flex items-center w-full">
        <div className="w-1/4">Wireframe</div>

        <div className="w-1/4">
          <CheckboxInput
            checked={material.wireframe}
            onChange={getEditMaterial("wireframe")}
          />
        </div>
      </div>

      <NumberField
        title="Opacity"
        value={material?.opacity ?? 1}
        step={0.1}
        min={0}
        max={1}
        onChange={getEditMaterial("opacity")}
      />

      <div className="flex items-center w-full">
        <div className="w-1/4">Texture</div>

        <div className="w-1/4">
          <ImageInput value={texture} onChange={handleTextureChange} />
        </div>

        {texture && (
          <div className="w-1/2 flex items-center space-x-2 pr-4">
            <div className="w-full overflow-hidden overflow-ellipsis">
              {texture.name}
            </div>
            <div onClick={handleDeleteTexture} className="hover:cursor-pointer">
              <IoMdClose />
            </div>
          </div>
        )}
      </div>

      <div className="flex items-center w-full">
        <div className="w-1/4">Color</div>

        <div className="w-1/4">
          <ColorInput
            id="color"
            value={material?.color}
            onChange={getEditMaterial("color")}
          />
        </div>
      </div>

      <div className="flex items-center w-full">
        <div className="w-1/4">Emissive</div>

        <div className="w-1/4">
          <ColorInput
            id="emissive"
            value={material.emissive}
            onChange={getEditMaterial("emissive")}
          />
        </div>
      </div>
    </div>
  );
}
