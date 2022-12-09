import { NodeIO } from "@gltf-transform/core";
import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
} from "lens";

import { lensClient } from "../lens";

export type AvatarStats = {
  materialCount: number;
  meshCount: number;
  skinCount: number;
  boneCount: number;
  polygonCount: number;
};

export async function getAvatarStats(
  publicationId: string
): Promise<AvatarStats> {
  // Fetch publication
  const { data } = await lensClient
    .query<GetPublicationQuery, GetPublicationQueryVariables>(
      GetPublicationDocument,
      {
        request: {
          publicationId,
        },
      }
    )
    .toPromise();

  const url = data?.publication?.metadata.media[1]?.original.url;
  if (!url) throw new Error("NOT_FOUND");

  // Fetch avatar
  const response = await fetch(url);
  const buffer = await response.arrayBuffer();
  const array = new Uint8Array(buffer);

  // Load avatar
  const io = new NodeIO();
  const document = await io.readBinary(array);

  // Get stats
  const materialCount = document.getRoot().listMaterials().length;
  const meshCount = document.getRoot().listMeshes().length;
  const skinCount = document.getRoot().listSkins().length;
  const boneCount = document
    .getRoot()
    .listSkins()
    .reduce((acc, skin) => acc + skin.listJoints().length, 0);
  const polygonCount = document
    .getRoot()
    .listMeshes()
    .reduce((acc, mesh) => {
      return (
        acc +
        mesh.listPrimitives().reduce((acc, primitive) => {
          const mode = primitive.getMode();
          if (mode !== 4) return acc;

          const indices = primitive.getIndices();
          if (indices) return acc + indices.getCount() / 3;

          const position = primitive.getAttribute("POSITION");
          if (position) return acc + position.getCount() / 3;

          return acc;
        }, 0)
      );
    }, 0);

  return {
    materialCount,
    meshCount,
    skinCount,
    boneCount,
    polygonCount,
  };
}
