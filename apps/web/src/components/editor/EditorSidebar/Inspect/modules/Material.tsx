import { ChangeEvent } from "react";

import { useStore } from "../../../../../helpers/editor/store";
import { getHandleChange } from "../helpers";

import ImageField from "../inputs/ImageField";

export default function Material() {
  const selected = useStore((state) => state.selected);
  const params = useStore(
    (state) => state.scene.instances[selected?.id]?.params
  );

  if (!params) return null;

  async function handleTextureChange(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files[0];
    const id = await useStore.getState().newTexture(file);

    getHandleChange("texture")(id);
  }

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Material</div>

      <ImageField
        title="Texture"
        value={params?.texture}
        onChange={handleTextureChange}
      />
    </div>
  );
}
