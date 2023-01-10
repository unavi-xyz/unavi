import { PointerData } from "../render/types";

export function getPointerData(event: PointerEvent, canvas: HTMLCanvasElement): PointerData {
  let pointer;
  if (canvas.ownerDocument.pointerLockElement) {
    pointer = {
      x: 0,
      y: 0,
      button: event.button,
    };
  } else {
    const rect = canvas.getBoundingClientRect();
    pointer = {
      x: ((event.clientX - rect.left) / rect.width) * 2 - 1,
      y: (-(event.clientY - rect.top) / rect.height) * 2 + 1,
      button: event.button,
    };
  }

  return {
    pointerId: event.pointerId,
    pointerType: event.pointerType,
    clientX: event.clientX,
    clientY: event.clientY,
    pageX: event.pageX,
    pageY: event.pageY,
    button: event.button,
    ctrlKey: event.ctrlKey,
    shiftKey: event.shiftKey,
    metaKey: event.metaKey,
    pointer,
  };
}
