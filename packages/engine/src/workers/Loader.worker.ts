import { GLTFLoader } from "../gltf/GLTFLoader";
import { FromLoaderLoadedGltf, ToLoaderMessage } from "../types";

onmessage = (event: MessageEvent<ToLoaderMessage>) => {
  const { subject, data, id } = event.data;

  switch (subject) {
    case "load_gltf":
      if (id === undefined) return;
      loadGltf(data.uri, id);
      break;
  }
};

async function loadGltf(uri: string, id: number) {
  const loader = new GLTFLoader();
  const data = await loader.load(uri);

  if (!data) return;

  const message: FromLoaderLoadedGltf = {
    id,
    subject: "loaded_gltf",
    data,
  };

  // Transferable objects
  const buffers = data.bufferViews?.map(({ bufferView }) => bufferView) ?? [];
  const images = data.images ?? [];

  postMessage(message, [...buffers, ...images]);
}
