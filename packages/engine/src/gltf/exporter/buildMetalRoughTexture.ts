import { LinearEncoding, Source, Texture, sRGBEncoding } from "three";

import { getCanvas } from "./getCanvas";

export function buildMetalRoughTexture(
  metalnessMap: Texture | null,
  roughnessMap: Texture | null
) {
  if (metalnessMap === roughnessMap) return metalnessMap;

  function getEncodingConversion(map: Texture) {
    if (map.encoding === sRGBEncoding) {
      return function SRGBToLinear(c: number) {
        return c < 0.04045
          ? c * 0.0773993808
          : Math.pow(c * 0.9478672986 + 0.0521327014, 2.4);
      };
    }

    return function LinearToLinear(c: number) {
      return c;
    };
  }

  const metalness = metalnessMap?.image;
  const roughness = roughnessMap?.image;

  const width = Math.max(metalness?.width || 0, roughness?.width || 0);
  const height = Math.max(metalness?.height || 0, roughness?.height || 0);

  const canvas = getCanvas();
  canvas.width = width;
  canvas.height = height;

  const context = canvas.getContext("2d");
  if (!context) throw new Error("Failed to get canvas context.");
  context.fillStyle = "#00ffff";
  context.fillRect(0, 0, width, height);

  const composite = context.getImageData(0, 0, width, height);

  if (metalness) {
    context.drawImage(metalness, 0, 0, width, height);

    const convert = getEncodingConversion(metalnessMap);
    const data = context.getImageData(0, 0, width, height).data;

    for (let i = 2; i < data.length; i += 4) {
      composite.data[i] = convert(data[i] / 256) * 256;
    }
  }

  if (roughness) {
    context.drawImage(roughness, 0, 0, width, height);

    const convert = getEncodingConversion(roughnessMap);
    const data = context.getImageData(0, 0, width, height).data;

    for (let i = 1; i < data.length; i += 4) {
      composite.data[i] = convert(data[i] / 256) * 256;
    }
  }

  context.putImageData(composite, 0, 0);

  const reference = metalnessMap || roughnessMap;

  if (!reference) throw new Error("Failed to find reference texture.");

  const texture = reference.clone();

  texture.source = new Source(canvas);
  texture.encoding = LinearEncoding;

  return texture;
}
