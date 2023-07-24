import { useClientStore } from "@unavi/engine";
import { BufferAttribute, BufferGeometry } from "three";

import { addMesh } from "../addMesh";

export function addThreeMesh(geometry: BufferGeometry) {
  const positionsAttr = geometry.getAttribute("position") as BufferAttribute;
  const normalsAttr = geometry.getAttribute("normal") as BufferAttribute;
  const uvsAttr = geometry.getAttribute("uv") as BufferAttribute;
  const indicesAttr = geometry.index as BufferAttribute;

  const positions = positionsAttr.array as Float32Array;
  const normals = normalsAttr.array as Float32Array;
  const uvs = uvsAttr.array as Float32Array;
  const indices = indicesAttr.array as Uint16Array;

  const indices32 = new Uint32Array(indices.length);

  for (let i = 0; i < indices.length; i++) {
    indices32[i] = indices[i] ?? 0;
  }

  const name = addMesh();

  useClientStore.getState().mirrorEvent({
    data: {
      indices: Array.from(indices32),
      normals: Array.from(normals),
      positions: Array.from(positions),
      target: name,
      uv: Array.from(uvs),
    },
    id: "xyz.unavi.editor.edit.mesh",
    target: "client",
  });

  return name;
}
