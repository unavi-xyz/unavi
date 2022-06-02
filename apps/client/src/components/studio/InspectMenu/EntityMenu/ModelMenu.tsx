import { MdOutlineFolderOpen } from "react-icons/md";

import { Entity } from "@wired-xr/scene";

import { useStudioStore } from "../../../../helpers/studio/store";
import MenuRow from "../MenuRow";

interface Props {
  selected: Entity<"Model">;
  handleChange: (key: string, value: any) => void;
}

export default function ModelMenu({ selected, handleChange }: Props) {
  async function handleSelectFile() {
    if (!selected?.id) return;

    try {
      //get file handle
      const [fileHandle] = await window.showOpenFilePicker({
        excludeAcceptAllOption: true,
        types: [
          {
            accept: {
              "model/gltf": [".gltf", ".glb"],
            },
          },
        ],
      });
      if (!fileHandle) return;

      //create asset
      const assetId = await useStudioStore
        .getState()
        .createAssetFromFile(fileHandle, "model");

      //set model
      handleChange("modelId", assetId);
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <MenuRow title="File">
      <div className="flex space-x-2 py-1">
        <button
          onClick={handleSelectFile}
          className="px-4 py-1 border rounded flex items-center justify-center transition
                   hover:bg-surfaceVariant cursor-default"
        >
          <MdOutlineFolderOpen />
        </button>
      </div>
    </MenuRow>
  );
}
