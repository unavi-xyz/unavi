import { GLTFLoader } from "../gltf/GLTFLoader";
import { LoadedGltf, ToGameMessage } from "../types";

onmessage = (event: MessageEvent<ToGameMessage>) => {
  const { type, data, id } = event.data;

  switch (type) {
    case "load_gltf":
      loadGltf(data.uri, id);
  }
};

async function loadGltf(uri: string, id: number) {
  const loader = new GLTFLoader();
  const data = await loader.load(uri);

  if (!data) return;

  const message: LoadedGltf = {
    id,
    type: "loaded_gltf",
    data,
  };

  // Transferable objects
  const buffers = data.bufferViews?.map(({ bufferView }) => bufferView) ?? [];
  const images = data.images ?? [];

  postMessage(message, [...buffers, ...images]);
}
