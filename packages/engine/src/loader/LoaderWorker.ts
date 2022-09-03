import { PostMessage, Transferable } from "../types";
import { GLTFLoader } from "./GLTFLoader";
import { FromLoaderMessage, ToLoaderMessage } from "./types";

export class LoaderWorker {
  #postMessage: PostMessage<FromLoaderMessage>;

  constructor(postMessage: PostMessage) {
    this.#postMessage = postMessage;

    postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToLoaderMessage>) => {
    const { subject, data, id } = event.data;

    switch (subject) {
      case "load_gltf":
        this.loadGltf(data, id);
        break;
    }
  };

  async loadGltf(uri: string, id?: number) {
    const loader = new GLTFLoader();
    const data = await loader.load(uri);
    const transfer: Transferable[] = [data.world];

    for (const image of data.assets.images) {
      transfer.push(image);
    }

    for (const accessor of data.assets.accessors) {
      transfer.push(accessor.buffer);
    }

    this.#postMessage({ subject: "gltf_loaded", data, id }, transfer);
  }
}
