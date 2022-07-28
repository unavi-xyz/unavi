export function canvasToBlobPromise(
  canvas: HTMLCanvasElement | OffscreenCanvas,
  mimeType: string
): Promise<Blob | null> {
  if (canvas instanceof OffscreenCanvas) {
    const quality = mimeType === "image/jpeg" ? 0.92 : mimeType === "image/webp" ? 0.8 : 1;
    return canvas.convertToBlob({
      type: mimeType,
      quality,
    });
  }

  return new Promise((resolve) => canvas.toBlob(resolve, mimeType));
}
