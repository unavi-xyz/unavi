import { ChangeEvent } from "react";
import { MdUploadFile } from "react-icons/md";
import { AssetName, ASSETS } from "3d";

import { useStore } from "../../helpers/store";

export default function GLTFCard() {
  async function handleChange(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files[0];
    const id = await useStore.getState().newGLTF(file);

    useStore
      .getState()
      .newInstance(AssetName.GLTF, { ...ASSETS.GLTF.params, model: id });

    e.target.value = null;
  }

  return (
    <div className="h-32">
      <label htmlFor="gltf-input">
        <div
          className="bg-neutral-100 hover:bg-neutral-200 p-4 h-full
                     rounded-xl hover:cursor-pointer flex flex-col justify-between"
        >
          <div className="text-2xl flex items-center justify-center h-full">
            <MdUploadFile />
          </div>
          GLTF Model
        </div>
      </label>

      <input
        id="gltf-input"
        type="file"
        accept=".gltf"
        onChange={handleChange}
        className="hidden"
      />
    </div>
  );
}
