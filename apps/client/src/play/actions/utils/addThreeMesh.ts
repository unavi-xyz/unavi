import { addMesh, editMesh } from "@unavi/engine";
import { BufferAttribute, BufferGeometry } from "three";

export function addThreeMesh(geometry: BufferGeometry) {
  const positionsAttr = geometry.getAttribute("position") as BufferAttribute;
  const normalsAttr = geometry.getAttribute("normal") as BufferAttribute;
  const uvsAttr = geometry.getAttribute("uv") as BufferAttribute;
  const indicesAttr = geometry.index as BufferAttribute;

  const positions = positionsAttr.array as Float32Array;
  const normals = normalsAttr.array as Float32Array;
  const uvs = uvsAttr.array as Float32Array;
  const indices = indicesAttr.array as Uint16Array;

  const id = addMesh();

  editMesh(id, {
    indices: Array.from(indices),
    normals: Array.from(normals),
    positions: Array.from(positions),
    uv: Array.from(uvs),
  });

  return id;
}
