import { GLTFLoader } from "../gltf/GLTFLoader";
import { FromGameLoadedGltf, ToGameMessage } from "../types";

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

  const message: FromGameLoadedGltf = {
    id,
    type: "loaded_gltf",
    data,
  };

  postMessage(message);
}
