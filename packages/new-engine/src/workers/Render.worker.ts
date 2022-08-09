import { RenderWorker } from "../renderer/RenderWorker";

const renderWorker = new RenderWorker();

renderWorker.postMessage = postMessage as any;
onmessage = renderWorker.onmessage;
