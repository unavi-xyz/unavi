import { Vec3 } from "engine";

export function rgbToHex(rgb: number[]): string {
  return `#${rgb.map((x) => x.toString(16).padStart(2, "0")).join("")}`;
}

export function hexToRgb(hex: string): Vec3 {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);

  if (!result || result[1] === undefined || result[2] === undefined || result[3] === undefined)
    return [0, 0, 0];

  return [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)];
}
