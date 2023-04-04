import { NodeIO } from "@gltf-transform/core";
import { AudioEmitter, AudioExtension, Avatar, AvatarExtension } from "@wired-labs/gltf-extensions";

/**
 * Gets the assets used by a model
 */
export async function getUsedAssets(model: Uint8Array) {
  // Load model
  const io = new NodeIO().registerExtensions([AudioExtension, AvatarExtension]);

  const doc = await io.readBinary(model);

  // Get used assets
  const usedAssets = new Set<string>();

  doc
    .getRoot()
    .listNodes()
    .forEach((node) => {
      const avatar = node.getExtension<Avatar>(Avatar.EXTENSION_NAME);
      if (avatar) usedAssets.add(avatar.getURI());

      const audioEmitter = node.getExtension<AudioEmitter>(AudioEmitter.EXTENSION_NAME);
      if (audioEmitter)
        audioEmitter.listSources().forEach((source) => {
          const audioData = source.getAudio();
          if (audioData) usedAssets.add(audioData.getURI());
        });
    });

  return usedAssets;
}
