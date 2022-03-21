import { ChangeEvent } from "react";
import { IoMdClose } from "react-icons/io";

import { useStore } from "../../../helpers/store";
import { useSections, handleChange } from "../helpers";

import { Select } from "../../../../base";
import ImageInput from "../inputs/ImageInput";
import ColorInput from "../inputs/ColorInput";
import NumberField from "../inputs/NumberField";
import CheckboxInput from "../inputs/CheckboxInput";

export default function Material() {
  const selected = useStore((state) => state.selected);
  const name = useStore((state) => state.scene.instances[selected?.id]?.name);
  const params = useStore(
    (state) => state.scene.instances[selected?.id]?.params
  );
  const texture = useStore(
    (state) => state.scene.textures[params?.material?.texture]
  );

  async function handleTextureChange(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files[0];
    e.target.value = null;

    const id = await useStore.getState().newTexture(file);
    const newMaterial = { ...params.material, texture: id };
    handleChange(newMaterial, "material");
  }

  function handleDeleteTexture() {
    const newMaterial = { ...params.material, texture: undefined };
    handleChange(newMaterial, "material");
  }

  function handleColorChange(e: ChangeEvent<HTMLInputElement>) {
    const color = e.target.value;
    const newMaterial = { ...params.material, color };
    handleChange(newMaterial, "material");
  }

  function handleEmissiveChange(e: ChangeEvent<HTMLInputElement>) {
    const emissive = e.target.value;
    const newMaterial = { ...params.material, emissive };
    handleChange(newMaterial, "material");
  }

  function handleSheenColorChange(e: ChangeEvent<HTMLInputElement>) {
    const sheenColor = e.target.value;
    const newMaterial = { ...params.material, sheenColor };
    handleChange(newMaterial, "material");
  }

  function handleOpacityChange(opacity: number) {
    const newMaterial = { ...params.material, opacity };
    handleChange(newMaterial, "material");
  }

  function handleReflectivityChange(reflectivity: number) {
    const newMaterial = { ...params.material, reflectivity };
    handleChange(newMaterial, "material");
  }

  function handleRoughnessChange(roughness: number) {
    const newMaterial = { ...params.material, roughness };
    handleChange(newMaterial, "material");
  }

  function handleMetalnessChange(metalness: number) {
    const newMaterial = { ...params.material, metalness };
    handleChange(newMaterial, "material");
  }

  function handleClearcoatChange(clearcoat: number) {
    const newMaterial = { ...params.material, clearcoat };
    handleChange(newMaterial, "material");
  }

  function handleSheenChange(sheen: number) {
    const newMaterial = { ...params.material, sheen };
    handleChange(newMaterial, "material");
  }

  function handleTypeChange(e: ChangeEvent<HTMLInputElement>) {
    const type = e.target.value;
    const newMaterial = { ...params.material, type };
    handleChange(newMaterial, "material");
  }

  function handleFlatShadingChange(e: ChangeEvent<HTMLInputElement>) {
    const flatShading = e.target.checked;
    const newMaterial = { ...params.material, flatShading };
    handleChange(newMaterial, "material");
  }

  const sections = useSections(name);

  if (!params || !sections.includes("material")) return null;

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Material</div>

      <div className="flex items-center w-full">
        <div className="w-1/4">Type</div>

        <div className="w-1/2">
          <Select
            value={params.material.type}
            options={["physical", "toon"]}
            onChange={handleTypeChange}
          />
        </div>
      </div>

      {params.material.type === "physical" && (
        <>
          <NumberField
            title="Reflectivity"
            value={params.material.reflectivity}
            step={0.1}
            min={0}
            max={1}
            onChange={handleReflectivityChange}
          />

          <NumberField
            title="Roughness"
            value={params.material.roughness}
            step={0.1}
            min={0}
            max={1}
            onChange={handleRoughnessChange}
          />

          <NumberField
            title="Metalness"
            value={params.material.metalness}
            step={0.1}
            min={0}
            max={1}
            onChange={handleMetalnessChange}
          />

          <NumberField
            title="Clearcoat"
            value={params.material.clearcoat}
            step={0.1}
            min={0}
            max={1}
            onChange={handleClearcoatChange}
          />

          <NumberField
            title="Sheen"
            value={params.material.sheen}
            step={0.1}
            min={0}
            max={1}
            onChange={handleSheenChange}
          />

          <div className="flex items-center w-full">
            <div className="w-1/4">Sheen Color</div>

            <div className="w-1/4">
              <ColorInput
                id="sheen"
                value={params.material.sheenColor}
                onChange={handleSheenColorChange}
              />
            </div>
          </div>

          <div className="flex items-center w-full">
            <div className="w-1/4">Flat Shading</div>

            <div className="w-1/4">
              <CheckboxInput
                checked={params?.material.flatShading}
                onChange={handleFlatShadingChange}
              />
            </div>
          </div>
        </>
      )}

      <div className="flex items-center w-full">
        <div className="w-1/4">Texture</div>

        <div className="w-1/4">
          <ImageInput value={texture?.value} onChange={handleTextureChange} />
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
            value={params.material?.color}
            onChange={handleColorChange}
          />
        </div>
      </div>

      <div className="flex items-center w-full">
        <div className="w-1/4">Emissive</div>

        <div className="w-1/4">
          <ColorInput
            id="emissive"
            value={params.material.emissive}
            onChange={handleEmissiveChange}
          />
        </div>
      </div>

      <NumberField
        title="Opacity"
        value={params.material?.opacity ?? 1}
        step={0.1}
        min={0}
        max={1}
        onChange={handleOpacityChange}
      />
    </div>
  );
}
