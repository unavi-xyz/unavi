export type PostMessage<M extends WorkerMessage = WorkerMessage> = (
  message: M,
  transfer?: Transferable[]
) => void;

export type Transferable = ArrayBuffer | MessagePort | ImageBitmap | OffscreenCanvas;

export type WorkerMessage<Subject extends string = string, Data = unknown> = {
  subject: Subject;
  data: Data;
};

export type Vec2 = [number, number];
export type Vec3 = [number, number, number];
export type Vec4 = [number, number, number, number];
