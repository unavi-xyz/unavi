import { NodeIO } from "@gltf-transform/core";
import { extensions } from "engine";

import createDecoderModule from "../../../public/scripts/draco_decoder";
import { getModelStats, ModelStats } from "../../utils/getModelStats";

export async function serverGetModelStats(url: string): Promise<ModelStats> {
  // Fetch model
  const response = await fetch(url);
  const buffer = await response.arrayBuffer();
  const array = new Uint8Array(buffer);

  // Get stats
  const io = new NodeIO()
    .registerExtensions(extensions)
    .registerDependencies({ "draco3d.decoder": await createDecoderModule() });

  return await getModelStats(io, array);
}
