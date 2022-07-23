import { GLTF } from "../schemaTypes";
import { AccessorResult } from "./loadAccessor";

export type SkinResult = {
  joints: number[];
  inverseBindMatrices: AccessorResult;
};

export async function loadSkin(
  index: number,
  json: GLTF,
  loadAccessor: (index: number) => Promise<AccessorResult>
): Promise<SkinResult> {
  if (json.skins === undefined) {
    throw new Error("No skins found");
  }

  const skinDef = json.skins[index];

  if (skinDef.inverseBindMatrices === undefined) {
    throw new Error("No meshes found");
  }

  const accessor = await loadAccessor(skinDef.inverseBindMatrices);

  const skinEntry = {
    joints: skinDef.joints,
    inverseBindMatrices: accessor,
  };

  return skinEntry;
}
