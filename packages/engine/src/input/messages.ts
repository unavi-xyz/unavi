import { WorkerMessage } from "../types";

export type PointerData = {
  pointerType: string;
  pointerId: number;
  pageX: number;
  pageY: number;
  clientX: number;
  clientY: number;
  button: number;
  ctrlKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
  pointer: {
    x: number;
    y: number;
    button: number;
  };
};

const subjects = [
  "mousemove",
  "wheel",
  "pointermove",
  "pointerup",
  "pointerdown",
  "pointercancel",
] as const;

type InputMessageSubject = (typeof subjects)[number];
type InputWorkerMessage<S extends InputMessageSubject, D> = WorkerMessage<S, D>;

export type InputMessage =
  | InputWorkerMessage<"mousemove", { x: number; y: number }>
  | InputWorkerMessage<"wheel", { deltaY: number }>
  | InputWorkerMessage<"pointermove", PointerData>
  | InputWorkerMessage<"pointerup", PointerData>
  | InputWorkerMessage<"pointerdown", PointerData>
  | InputWorkerMessage<"pointercancel", PointerData>;

export function isInputMessage(message: WorkerMessage): message is InputMessage {
  return subjects.includes(message.subject as InputMessageSubject);
}
