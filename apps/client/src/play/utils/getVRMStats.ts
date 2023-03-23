import { NodeIO } from "@gltf-transform/core";
import {
  VRM,
  VRM0,
  VRM0MetadataExtension,
  VRMMetadataExtension,
} from "@wired-labs/gltf-extensions";
import { extensions } from "engine";

import { getModelStats, ModelStats } from "../../utils/getModelStats";

export interface VRMStats extends ModelStats {
  name: string;
}

export async function getVRMStats(url: string): Promise<VRMStats> {
  // Fetch model
  const response = await fetch(url);
  const buffer = await response.arrayBuffer();
  const array = new Uint8Array(buffer);

  // Read model
  const io = new NodeIO().registerExtensions([
    ...extensions,
    VRMMetadataExtension,
    VRM0MetadataExtension,
  ]);
  const doc = await io.readBinary(array);

  // Get stats
  const stats = await getModelStats(doc, array);

  // Get name
  const vrm = doc.getRoot().getExtension<VRM>(VRM.EXTENSION_NAME);
  const vrm0 = doc.getRoot().getExtension<VRM0>(VRM0.EXTENSION_NAME);
  const name = vrm ? vrm.getMeta().name : vrm0 ? vrm0.getMeta().title : "";

  return {
    ...stats,
    name,
  };
}
