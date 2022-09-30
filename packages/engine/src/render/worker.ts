import { RenderWorker } from "./RenderWorker";

const renderWorker = new RenderWorker(postMessage.bind(this) as any);
onmessage = renderWorker.onmessage;
