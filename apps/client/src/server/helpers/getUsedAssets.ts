import { NodeIO } from "@gltf-transform/core";
import { Avatar, AvatarExtension } from "@wired-labs/gltf-extensions";

/**
 * Gets the assets used by a model
 */
export async function getUsedAssets(model: Uint8Array) {
  // Load model
  const io = new NodeIO().registerExtensions([AvatarExtension]);

  const doc = await io.readBinary(model);

  // Get used assets
  const usedAssets = new Set<string>();

  doc
    .getRoot()
    .listNodes()
    .forEach((node) => {
      const avatar = node.getExtension<Avatar>(Avatar.EXTENSION_NAME);
      if (!avatar) return;
      usedAssets.add(avatar.getURI());
    });

  return usedAssets;
}
