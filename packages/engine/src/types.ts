import { BaseEvent } from "property-graph";

export type PostMessage<M extends MessageJSON = MessageJSON> = (
  message: M,
  transfer?: Transferable[]
) => void;

export type Transferable = ArrayBuffer | MessagePort | ImageBitmap | OffscreenCanvas;

export type MessageJSON<Subject extends string = string, Data = unknown> = {
  subject: Subject;
  data: Data;
};

export interface MessageEvent<Type extends string = string, Data = unknown> extends BaseEvent {
  type: Type;
  data: Data;
}

export type Vec2 = [number, number];
export type Vec3 = [number, number, number];
export type Vec4 = [number, number, number, number];
