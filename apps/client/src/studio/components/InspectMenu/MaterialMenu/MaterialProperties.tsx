import { WritableDraft } from "immer/dist/internal";
import { ChangeEvent, useRef } from "react";
import { MdClose } from "react-icons/md";

import { IEntity, IMaterial } from "@wired-xr/engine";

import { useStudioStore } from "../../../../studio/store";
import ColorInput from "../../../../ui/base/ColorInput";
import Select from "../../../../ui/base/Select";
import { round } from "../../../../utils/round";
import {
  useAsset,
  useImageAsset,
  useMaterialAsset,
} from "../../../hooks/useAsset";
import MenuRow from "../MenuRow";
import NumberInput from "../NumberInput";

interface Props {
  selected: IEntity;
}

export default function MaterialProperties({ selected }: Props) {
  //@ts-ignore
  const materialId = selected?.props?.materialId;
  const materialAsset = useMaterialAsset(materialId);
  const material = materialAsset?.data;
  const textureAsset = useImageAsset(material?.textureId);

  const colorRef = useRef<HTMLInputElement>(null);
  const emissiveRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<any>(null);

  function updateMaterial(
    callback: (draft: WritableDraft<IMaterial>) => void,
    delay = 0
  ) {
    const debounce = setTimeout(async () => {
      try {
        //update material
        await useStudioStore
          .getState()
          .updateMaterialFile(materialId, callback);

        //load material
        await useStudioStore.getState().loadAsset(materialId, true);
      } catch (error) {
        console.error(error);
      }
    }, delay);

    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = debounce;
  }

  function onColorChange() {
    //if same color, do nothing
    if (colorRef.current?.value === material?.color) return;

    updateMaterial((draft) => {
      const newColor = colorRef.current?.value;
      if (!material || !newColor) return;
      draft.color = newColor;
    }, 200);
  }

  function onEmissiveChange() {
    //if same color, do nothing
    if (emissiveRef.current?.value === material?.emissive) return;

    updateMaterial((draft) => {
      const newEmissive = emissiveRef.current?.value;
      if (!material || !newEmissive) return;
      draft.emissive = newEmissive;
    }, 200);
  }

  async function handleOpacityChange(value: string) {
    if (isNaN(Number(value)) || value === "" || !selected) return;
    const rounded = Math.min(Math.max(round(Number(value)), 0), 1);

    updateMaterial((draft) => {
      draft.opacity = rounded;
    });
  }

  async function handleRoughnessChange(value: string) {
    if (isNaN(Number(value)) || value === "" || !selected) return;
    const rounded = Math.min(Math.max(round(Number(value)), 0), 1);

    updateMaterial((draft) => {
      draft.roughness = rounded;
    });
  }

  async function handleMetalnessChange(value: string) {
    if (isNaN(Number(value)) || value === "" || !selected) return;
    const rounded = Math.min(Math.max(round(Number(value)), 0), 1);

    updateMaterial((draft) => {
      draft.metalness = rounded;
    });
  }

  async function handleTextureClick() {
    try {
      //select image
      const [fileHandle] = await window.showOpenFilePicker({
        types: [
          {
            accept: { "image/*": [".png", ".jpg", ".jpeg", ".gif"] },
          },
        ],
      });

      //create image asset
      const assetId = await useStudioStore
        .getState()
        .createAssetFromFile(fileHandle, "image");
      if (!assetId) return;

      //update material
      updateMaterial((draft) => {
        draft.textureId = assetId;
      });
    } catch (error) {
      console.error(error);
    }
  }

  async function handleRemoveTexture() {
    updateMaterial((draft) => {
      draft.textureId = undefined;
    });
  }

  if (!material) return null;

  return (
    <>
      <MenuRow title="Color">
        <div className="h-6">
          <ColorInput
            inputRef={colorRef}
            defaultValue={material.color}
            onChange={onColorChange}
          />
        </div>
      </MenuRow>

      <MenuRow title="Emissive">
        <div className="h-6">
          <ColorInput
            inputRef={emissiveRef}
            defaultValue={material.emissive}
            onChange={onEmissiveChange}
          />
        </div>
      </MenuRow>

      <MenuRow title="Opacity">
        <NumberInput
          updatedValue={String(round(material.opacity))}
          onChange={(e) => handleOpacityChange(e.currentTarget.value)}
        />
      </MenuRow>

      <MenuRow title="Roughness">
        <NumberInput
          updatedValue={String(round(material.roughness))}
          onChange={(e) => handleRoughnessChange(e.currentTarget.value)}
        />
      </MenuRow>

      <MenuRow title="Metalness">
        <NumberInput
          updatedValue={String(round(material.metalness))}
          onChange={(e) => handleMetalnessChange(e.currentTarget.value)}
        />
      </MenuRow>

      <MenuRow title="FlatShading">
        <input
          type="checkbox"
          checked={material.flatShading}
          onChange={(e) => {
            updateMaterial((draft) => {
              draft.flatShading = !e.target.checked;
            });
          }}
        />
      </MenuRow>

      <MenuRow title="Side">
        <Select
          defaultValue={material.side}
          options={["Front", "Back", "Double"]}
          thin
          onChange={(e: ChangeEvent<HTMLSelectElement>) =>
            updateMaterial((draft) => {
              draft.side = e.target.value as IMaterial["side"];
            })
          }
        />
      </MenuRow>

      <MenuRow title="Texture">
        <div className="w-full flex relative">
          <div onClick={handleTextureClick} className="border w-12 h-6 rounded">
            {textureAsset?.data && (
              <img
                src={textureAsset.data}
                alt="texture preview"
                className="w-full h-full object-cover rounded"
              />
            )}
          </div>

          {material?.textureId && (
            <div className="absolute right-2 h-full">
              <button
                onClick={handleRemoveTexture}
                className="h-full cursor-default text-outline hover:text-inherit transition p-1"
              >
                <MdClose className="h-full" />
              </button>
            </div>
          )}
        </div>
      </MenuRow>
    </>
  );
}
