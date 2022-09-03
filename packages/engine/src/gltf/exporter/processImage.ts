import { RGBAFormat } from "three";

import { GLTFExporterOptions } from "../GLTFExporter";
import { GLTF, Image } from "../schemaTypes";
import { canvasToBlobPromise } from "./canvasToBlobPromise";
import { getCanvas } from "./getCanvas";

const maxTextureSize = Infinity;

export async function processImage(
  image: any,
  format: number,
  flipY: boolean,
  mimeType: string,
  json: GLTF,
  options: GLTFExporterOptions,
  processBufferViewImage: (blob: Blob) => Promise<number>
): Promise<number> {
  const imageDef: Image = { mimeType };

  // Create canvas
  const canvas = getCanvas();

  canvas.width = Math.min(image.width, maxTextureSize);
  canvas.height = Math.min(image.height, maxTextureSize);

  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Could not get context");

  if (flipY === true) {
    ctx.translate(0, canvas.height);
    ctx.scale(1, -1);
  }

  // Draw image to canvas
  if (image.data !== undefined) {
    // DataTexture
    if (format !== RGBAFormat) {
      throw new Error("DataTexture format must be RGBAFormat");
    }

    if (image.width > maxTextureSize || image.height > maxTextureSize) {
      throw new Error(
        "DataTexture width and height must not exceed " + maxTextureSize
      );
    }

    const data = new Uint8ClampedArray(image.height * image.width * 4);

    for (let i = 0; i < image.data.length; i += 4) {
      data[i] = image.data[i];
      data[i + 1] = image.data[i + 1];
      data[i + 2] = image.data[i + 2];
      data[i + 3] = image.data[i + 3];
    }

    ctx.putImageData(new ImageData(data, image.width, image.height), 0, 0);
  } else {
    ctx.drawImage(image, 0, 0, canvas.width, canvas.height);
  }

  // Convert canvas to image uri
  if (options.binary) {
    // Write image to buffer
    const blob = await canvasToBlobPromise(canvas, mimeType);
    if (!blob) throw new Error("Could not get blob");

    const index = await processBufferViewImage(blob);
    imageDef.bufferView = index;
  } else {
    // Create data url
    if (canvas instanceof HTMLCanvasElement) {
      imageDef.uri = canvas.toDataURL(mimeType);
    } else {
      const blob = await canvasToBlobPromise(canvas, mimeType);
      if (!blob) throw new Error("Could not get blob");

      const reader = new FileReader();
      reader.readAsDataURL(blob);
      reader.onloadend = () => {
        const result = reader.result as string;
        imageDef.uri = result;
      };
    }
  }

  if (!json.images) json.images = [];
  const index = json.images.push(imageDef) - 1;
  return index;
}
