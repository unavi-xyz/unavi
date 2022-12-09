import { PointerData, ToRenderMessage, WheelData } from "../types";

export type FakeKeyDownEvent = CustomEvent<{ code: string }>;
export type FakePointerEvent = CustomEvent<PointerData>;
export type FakeWheelEvent = CustomEvent<WheelData>;

export interface RenderPlugin {
  onmessage: (event: MessageEvent<ToRenderMessage>) => void;
  update?: (delta: number) => void;
  destroy?: () => void;
}
