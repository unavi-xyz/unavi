export function getCanvas() {
  if (
    typeof document === "undefined" &&
    typeof OffscreenCanvas !== "undefined"
  ) {
    return new OffscreenCanvas(1, 1);
  }

  return document.createElement("canvas");
}
