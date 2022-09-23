export type PostMessage<M extends WorkerMessage = WorkerMessage> = (
  message: M,
  transfer?: Transferable[]
) => void;

export type Transferable =
  | ArrayBuffer
  | MessagePort
  | ImageBitmap
  | OffscreenCanvas;

export type WorkerMessage<Subject extends string = string, Data = any> = {
  subject: Subject;
  data: Data;
};

export type Triplet = [number, number, number];
