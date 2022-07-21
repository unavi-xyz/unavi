import { BufferView, GLTF } from "./schemaTypes";

export type LoadedGLTF = {
  json: GLTF;
  bufferViews?: LoadedBufferView[];
  images?: ImageBitmap[];
};

export type LoadedBufferView = {
  bufferView: ArrayBuffer;
  bufferViewDef: BufferView;
};
