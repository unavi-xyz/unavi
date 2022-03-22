import { ChangeEvent } from "react";
import { IoMdClose } from "react-icons/io";
import { Material, Material as Material3d } from "3d";

import { sceneManager, useStore } from "../../../../helpers/store";

import { Select } from "../../../../../base";
import ImageInput from "../../inputs/ImageInput";
import ColorInput from "../../inputs/ColorInput";
import NumberField from "../../inputs/NumberField";
import CheckboxInput from "../../inputs/CheckboxInput";

export default function EditMaterial() {
  const selected = useStore((state) => state.selected);
  const properties = useStore(
    (state) => state.scene.instances[selected?.id]?.properties
  );
  const assets = useStore((state) => state.scene.assets);
  const materials = useStore((state) => state.scene.materials);

  if (!properties || !("material" in properties)) return null;

  const material = materials[properties.material];
  const texture = assets[material?.texture];

  if (!material) return null;

  function updateMaterial(changes: Partial<Material>) {
    if (!("material" in properties)) return;
    sceneManager.editMaterial(properties.material, changes);
  }

  async function handleTextureChange(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files[0];
    e.target.value = null;

    const id = await sceneManager.newAsset(file);
    updateMaterial({ texture: id });
  }

  function editMaterial<T extends keyof Material3d>(
    key: T,
    value: Material3d[T]
  ) {
    updateMaterial({ [key]: value });
  }

  function getEditMaterial(key: keyof Material3d) {
    return (value: Material3d[typeof key]) =>
      editMaterial<typeof key>(key, value);
  }

  function handleDeleteTexture() {
    sceneManager.deleteAsset(material.texture);
  }

  function handleColorChange(e: ChangeEvent<HTMLInputElement>) {
    const color = e.target.value;
    updateMaterial({ color });
  }

  function handleEmissiveChange(e: ChangeEvent<HTMLInputElement>) {
    const emissive = e.target.value;
    updateMaterial({ emissive });
  }

  function handleSheenColorChange(e: ChangeEvent<HTMLInputElement>) {
    const sheenColor = e.target.value;
    updateMaterial({ sheenColor });
  }

  function handleTypeChange(e: ChangeEvent<HTMLInputElement>) {
    const type = e.target.value as any;
    updateMaterial({ type });
  }

  function handleFlatShadingChange(e: ChangeEvent<HTMLInputElement>) {
    const flatShading = e.target.checked;
    updateMaterial({ flatShading });
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center w-full">
        <div className="w-1/4">Type</div>

        <div className="w-1/2">
          <Select
            value={material.type}
            options={["physical", "toon"]}
            onChange={handleTypeChange}
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
                id="sheen"
                value={material.sheenColor}
                onChange={handleSheenColorChange}
              />
            </div>
          </div>

          <div className="flex items-center w-full">
            <div className="w-1/4">Flat Shading</div>

            <div className="w-1/4">
              <CheckboxInput
                checked={material.flatShading}
                onChange={handleFlatShadingChange}
              />
            </div>
          </div>
        </>
      )}

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
            onChange={handleColorChange}
          />
        </div>
      </div>

      <div className="flex items-center w-full">
        <div className="w-1/4">Emissive</div>

        <div className="w-1/4">
          <ColorInput
            id="emissive"
            value={material.emissive}
            onChange={handleEmissiveChange}
          />
        </div>
      </div>
    </div>
  );
}
