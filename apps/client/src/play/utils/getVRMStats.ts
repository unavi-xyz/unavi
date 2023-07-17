import { NodeIO } from "@gltf-transform/core";
import { KHRDracoMeshCompression } from "@gltf-transform/extensions";
import {
  VRM,
  VRM0,
  VRM0MetadataExtension,
  VRMMetadataExtension,
} from "@unavi/gltf-extensions";

import { getModelStats, ModelStats } from "./getModelStats";

const cache = new Map<string, VRMStats>();

export interface VRMStats extends ModelStats {
  name: string;
}

export async function getVRMStats(url: string): Promise<VRMStats> {
  // Check cache
  const cached = cache.get(url);
  if (cached) return cached;

  // Fetch model
  const response = await fetch(url);
  const buffer = await response.arrayBuffer();
  const array = new Uint8Array(buffer);

  // Read model
  const io = new NodeIO().registerExtensions([
    KHRDracoMeshCompression,
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

  const data: VRMStats = { ...stats, name };

  // Set cache
  cache.set(url, data);

  return data;
}
