import { RenderWorker } from "../render/RenderWorker";

//@ts-ignore
const renderWorker = new RenderWorker(postMessage.bind(this));
onmessage = renderWorker.onmessage;
