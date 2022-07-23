import { BufferViewResult } from "./loader/loadBufferView";
import { GLTF } from "./schemaTypes";

export type LoadedGLTF = {
  json: GLTF;
  bufferViews?: BufferViewResult[];
  images?: ImageBitmap[];
};
