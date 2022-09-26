import { PostMessage, Transferable } from "../types";
import { GLTFLoader } from "./GLTFLoader";
import { FromLoaderMessage, ToLoaderMessage } from "./types";

/*
 * Loads heavy assets in a separate thread, then sends them to the main thread.
 */
export class LoaderWorker {
  #postMessage: PostMessage<FromLoaderMessage>;

  constructor(postMessage: PostMessage) {
    this.#postMessage = postMessage;
    postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToLoaderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "load_gltf":
        this.loadGltf(data.id, data.uri);
        break;
    }
  };

  async loadGltf(id: string, uri: string) {
    // Load the glTF
    const loader = new GLTFLoader();
    const scene = await loader.load(uri);
    const sceneJSON = scene.toJSON(true);

    // Send to main thread
    const buffers = new Set<ArrayBuffer>();
    sceneJSON.accessors.forEach((accessor) => {
      const buffer = accessor.array.buffer;
      buffers.add(buffer);
    });

    const bitmaps = sceneJSON.images.map((image) => image.bitmap);

    const transfer: Transferable[] = [...buffers, ...bitmaps];

    this.#postMessage(
      {
        subject: "gltf_loaded",
        data: {
          id,
          scene: sceneJSON,
        },
      },
      transfer
    );
  }
}
