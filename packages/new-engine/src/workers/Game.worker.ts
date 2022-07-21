import { GLTFLoader } from "../gltf/GLTFLoader";
import { GameMessage, MainGLTFMessage } from "../types";

onmessage = (event: MessageEvent<GameMessage>) => {
  const { type, data } = event.data;

  switch (type) {
    case "loadGltf":
      loadGltf(data.url);
  }
};

async function loadGltf(url: string) {
  const loader = new GLTFLoader();
  const data = await loader.load(url);

  if (!data) return;

  const message: MainGLTFMessage = {
    type: "gltf",
    data,
  };

  postMessage(message);
}
