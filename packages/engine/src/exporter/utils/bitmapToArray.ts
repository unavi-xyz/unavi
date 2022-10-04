export function bitmapToArray(image: ImageBitmap): Uint8Array {
  const canvas = document.createElement("canvas");
  canvas.width = image.width;
  canvas.height = image.height;

  const context = canvas.getContext("2d");
  if (!context) throw new Error("Could not create canvas context");

  context.drawImage(image, 0, 0);

  const { data } = context.getImageData(0, 0, image.width, image.height);
  return new Uint8Array(data.buffer);
}
