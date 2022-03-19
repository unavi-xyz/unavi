import { ChangeEvent } from "react";
import { IoMdClose } from "react-icons/io";

import { useStore } from "../../../helpers/store";
import { useSections, handleChange } from "../helpers";

import ImageInput from "../inputs/ImageInput";
import ColorInput from "../inputs/ColorInput";
import NumberField from "../inputs/NumberField";

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

  function handleOpacityChange(opacity: number) {
    const newMaterial = { ...params.material, opacity };
    handleChange(newMaterial, "material");
  }

  const sections = useSections(name);

  if (!params || !sections.includes("material")) return null;

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Material</div>

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
            value={params.material?.color}
            onChange={handleColorChange}
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
