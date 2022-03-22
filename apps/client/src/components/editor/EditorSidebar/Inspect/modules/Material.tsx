import { ChangeEvent } from "react";
import { IoMdClose } from "react-icons/io";
import { Material as Material3d } from "3d";

import { sceneManager, useStore } from "../../../helpers/store";
import { handleChange } from "../helpers";

import { Select } from "../../../../base";
import ImageInput from "../inputs/ImageInput";
import ColorInput from "../inputs/ColorInput";
import NumberField from "../inputs/NumberField";
import CheckboxInput from "../inputs/CheckboxInput";

export default function Material() {
  const selected = useStore((state) => state.selected);
  const properties = useStore(
    (state) => state.scene.instances[selected?.id]?.properties
  );
  const assets = useStore((state) => state.scene.assets);

  if (!properties || !("material" in properties)) return null;

  const texture = assets[properties.material?.texture];

  async function handleTextureChange(e: ChangeEvent<HTMLInputElement>) {
    if (!("material" in properties)) return;
    const file = e.target.files[0];
    e.target.value = null;

    const id = await sceneManager.newAsset(file);
    const newMaterial = { ...properties.material, texture: id };
    handleChange(newMaterial, "material");
  }

  function editMaterial<T extends keyof Material3d>(
    key: T,
    value: Material3d[T]
  ) {
    if (!("material" in properties)) return;
    const newMaterial = { ...properties.material, [key]: value };
    handleChange(newMaterial, "material");
  }

  function getEditMaterial(key: keyof Material3d) {
    return (value: Material3d[typeof key]) =>
      editMaterial<typeof key>(key, value);
  }

  function handleDeleteTexture() {
    if (!("material" in properties)) return;
    sceneManager.deleteAsset(properties.material.texture);
  }

  function handleColorChange(e: ChangeEvent<HTMLInputElement>) {
    const color = e.target.value;
    if (!("material" in properties)) return;
    const newMaterial = { ...properties.material, color };
    handleChange(newMaterial, "material");
  }

  function handleEmissiveChange(e: ChangeEvent<HTMLInputElement>) {
    const emissive = e.target.value;
    if (!("material" in properties)) return;
    const newMaterial = { ...properties.material, emissive };
    handleChange(newMaterial, "material");
  }

  function handleSheenColorChange(e: ChangeEvent<HTMLInputElement>) {
    const sheenColor = e.target.value;
    if (!("material" in properties)) return;
    const newMaterial = { ...properties.material, sheenColor };
    handleChange(newMaterial, "material");
  }

  function handleTypeChange(e: ChangeEvent<HTMLInputElement>) {
    const type = e.target.value;
    if (!("material" in properties)) return;
    const newMaterial = { ...properties.material, type };
    handleChange(newMaterial, "material");
  }

  function handleFlatShadingChange(e: ChangeEvent<HTMLInputElement>) {
    const flatShading = e.target.checked;
    if (!("material" in properties)) return;
    const newMaterial = { ...properties.material, flatShading };
    handleChange(newMaterial, "material");
  }

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Material</div>

      <div className="flex items-center w-full">
        <div className="w-1/4">Type</div>

        <div className="w-1/2">
          <Select
            value={properties.material.type}
            options={["physical", "toon"]}
            onChange={handleTypeChange}
          />
        </div>
      </div>

      {properties.material.type === "physical" && (
        <>
          <NumberField
            title="Reflectivity"
            value={properties.material.reflectivity}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("reflectivity")}
          />

          <NumberField
            title="Roughness"
            value={properties.material.roughness}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("roughness")}
          />

          <NumberField
            title="Metalness"
            value={properties.material.metalness}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("metalness")}
          />

          <NumberField
            title="Clearcoat"
            value={properties.material.clearcoat}
            step={0.1}
            min={0}
            max={1}
            onChange={getEditMaterial("clearcoat")}
          />

          <NumberField
            title="Sheen"
            value={properties.material.sheen}
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
                value={properties.material.sheenColor}
                onChange={handleSheenColorChange}
              />
            </div>
          </div>

          <div className="flex items-center w-full">
            <div className="w-1/4">Flat Shading</div>

            <div className="w-1/4">
              <CheckboxInput
                checked={properties?.material.flatShading}
                onChange={handleFlatShadingChange}
              />
            </div>
          </div>
        </>
      )}

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
            value={properties.material?.color}
            onChange={handleColorChange}
          />
        </div>
      </div>

      <div className="flex items-center w-full">
        <div className="w-1/4">Emissive</div>

        <div className="w-1/4">
          <ColorInput
            id="emissive"
            value={properties.material.emissive}
            onChange={handleEmissiveChange}
          />
        </div>
      </div>

      <NumberField
        title="Opacity"
        value={properties.material?.opacity ?? 1}
        step={0.1}
        min={0}
        max={1}
        onChange={getEditMaterial("opacity")}
      />
    </div>
  );
}
