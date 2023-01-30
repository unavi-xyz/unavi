import { PostMessage } from "../types";
import { RenderThread } from "./RenderThread";

const renderThread = new RenderThread(postMessage.bind(this) as PostMessage);

onmessage = renderThread.onmessage;
