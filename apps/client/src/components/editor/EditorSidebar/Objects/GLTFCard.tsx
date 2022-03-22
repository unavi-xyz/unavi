import { ChangeEvent } from "react";
import { MdUploadFile } from "react-icons/md";

import { sceneManager, useStore } from "../../helpers/store";

export default function GLTFCard() {
  async function handleChange(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files[0];
    const src = await sceneManager.newAsset(file);

    const id = sceneManager.newInstance("GLTF");
    sceneManager.editInstance(id, { src });

    e.target.value = null;
  }

  return (
    <div className="h-32">
      <label htmlFor="gltf-input">
        <div
          className="p-4 h-full rounded-2xl space-y-1 hover:shadow-lg transition-all duration-300
                     hover:cursor-pointer flex flex-col items-center justify-center"
        >
          <div className="text-2xl">
            <MdUploadFile />
          </div>

          <div className="w-full flex justify-center">GLTF Model</div>
        </div>
      </label>

      <input
        id="gltf-input"
        type="file"
        accept=".gltf,.glb"
        onChange={handleChange}
        className="hidden"
      />
    </div>
  );
}
