import { NodeIO } from "@gltf-transform/core";
import { extensions } from "engine";

import { getModelStats, ModelStats } from "../../utils/getModelStats";

export async function clientGetModelStats(url: string): Promise<ModelStats> {
  // Fetch model
  const response = await fetch(url);
  const buffer = await response.arrayBuffer();
  const array = new Uint8Array(buffer);

  // Get stats
  const io = new NodeIO().registerExtensions(extensions).registerDependencies({
    // @ts-ignore
    "draco3d.decoder": await new DracoDecoderModule(),
  });

  return await getModelStats(io, array);
}
