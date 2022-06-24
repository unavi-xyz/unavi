import { MdClose, MdOutlineFolderOpen } from "react-icons/md";

import { IEntity } from "@wired-xr/engine";

import { useAssetName } from "../../../../helpers/studio/hooks/useAssetName";
import { useStudioStore } from "../../../../helpers/studio/store";

interface Props {
  selected: IEntity<"Model">;
  handleChange: (key: string, value: any) => void;
}

export default function ModelMenu({ selected, handleChange }: Props) {
  //@ts-ignore
  const modelId = selected.props?.modelId;
  const modelName = useAssetName(modelId);

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

  async function handleRemoveModel() {
    handleChange("modelId", undefined);
  }

  return (
    <div className="flex space-x-2 py-1">
      <button
        onClick={handleSelectFile}
        className="px-4 py-1 border rounded flex items-center justify-center transition
             hover:bg-surfaceVariant cursor-default"
      >
        <MdOutlineFolderOpen />
      </button>

      {modelId ? (
        <div className="w-full flex relative">
          <div>{modelName}</div>
          <div className="absolute right-2 h-full">
            <button
              onClick={handleRemoveModel}
              className="h-full cursor-default text-outline hover:text-inherit transition p-1"
            >
              <MdClose className="h-full" />
            </button>
          </div>
        </div>
      ) : (
        <button
          onClick={handleSelectFile}
          className="w-full flex items-center justify-center border rounded space-x-1
                 cursor-default transition hover:bg-surfaceVariant"
        >
          <div>Select GLTF Model</div>
        </button>
      )}
    </div>
  );
}
