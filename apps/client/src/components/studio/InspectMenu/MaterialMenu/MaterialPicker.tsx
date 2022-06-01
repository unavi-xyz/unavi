import { MdClose, MdOutlineAdd, MdOutlineFolderOpen } from "react-icons/md";

import { Entity, Material } from "@wired-xr/scene";

import { useAssetName } from "../../../../helpers/studio/hooks/useAssetName";
import { useStudioStore } from "../../../../helpers/studio/store";

interface Props {
  selected: Entity;
}

export default function MaterialPicker({ selected }: Props) {
  //@ts-ignore
  const materialId = selected?.props?.materialId;
  const materialName = useAssetName(materialId);

  async function handleSelectMaterial() {
    if (!selected?.id) return;

    try {
      //get file handle
      const [fileHandle] = await window.showOpenFilePicker({
        excludeAcceptAllOption: true,
        types: [
          {
            accept: {
              "application/json": [".json"],
            },
          },
        ],
      });
      if (!fileHandle) return;

      //set material
      useStudioStore.getState().setMaterialFromFile(selected.id, fileHandle);
    } catch (error) {
      console.error(error);
    }
  }

  async function handleNewMaterial() {
    if (!selected?.id) return;

    const newMaterial: Material = {
      color: "#ffffff",
      emissive: "#000000",
      opacity: 1,
      roughness: 1,
      metalness: 0,
      flatShading: false,
      textureId: undefined,
    };

    try {
      //create material file
      const fileHandle = await window.showSaveFilePicker({
        suggestedName: "Material001",
        excludeAcceptAllOption: true,
        types: [
          {
            accept: {
              "application/json": [".json"],
            },
          },
        ],
      });

      //write material to file
      const writableStream = await fileHandle.createWritable();
      await writableStream.write(JSON.stringify(newMaterial, null, 2));
      await writableStream.close();

      //set material
      await useStudioStore
        .getState()
        .setMaterialFromFile(selected.id, fileHandle);
    } catch (error) {
      console.error(error);
    }
  }

  async function handleRemoveMaterial() {
    if (!selected?.id) return;

    useStudioStore.getState().updateEntity(selected.id, (draft) => {
      //@ts-ignore
      draft.props.materialId = undefined;
    });
  }

  return (
    <div className="flex space-x-2 py-1">
      <button
        onClick={handleSelectMaterial}
        className="px-4 py-1 border rounded flex items-center justify-center transition
                 hover:bg-surfaceVariant cursor-default"
      >
        <MdOutlineFolderOpen />
      </button>

      {materialId ? (
        <div className="w-full flex relative">
          <div>{materialName}</div>
          <div className="absolute right-2 h-full">
            <button
              onClick={handleRemoveMaterial}
              className="h-full cursor-default text-outline hover:text-inherit transition p-1"
            >
              <MdClose className="h-full" />
            </button>
          </div>
        </div>
      ) : (
        <button
          onClick={handleNewMaterial}
          className="w-full flex items-center justify-center border rounded space-x-1
                     cursor-default transition hover:bg-surfaceVariant"
        >
          <MdOutlineAdd />
          <div>New Material</div>
        </button>
      )}
    </div>
  );
}
