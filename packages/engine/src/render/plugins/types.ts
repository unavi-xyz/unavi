import { PointerData, WheelData } from "../types";

export type FakeKeyDownEvent = CustomEvent<{ code: string }>;
export type FakePointerEvent = CustomEvent<PointerData>;
export type FakeWheelEvent = CustomEvent<WheelData>;
