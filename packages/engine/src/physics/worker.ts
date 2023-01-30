import { PostMessage } from "../types";
import { PhysicsThread } from "./PhysicsThread";

const physicsThread = new PhysicsThread(postMessage.bind(this) as PostMessage);

onmessage = physicsThread.onmessage;
