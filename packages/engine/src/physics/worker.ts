import { PhysicsWorker } from "./PhysicsWorker";

// @ts-ignore
const physicsWorker = new PhysicsWorker(postMessage.bind(this));
onmessage = physicsWorker.onmessage;
