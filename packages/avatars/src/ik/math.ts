export function cos_sss(a: number, b: number, c: number) {
  // Law of Cosines - SSS : cos(C) = (a^2 + b^2 - c^2) / 2ab
  // The Angle between A and B with C being the opposite length of the angle.
  let value = (a * a + b * b - c * c) / (2 * a * b);
  // Clamp to prevent NaN Errors
  if (value < -1) value = -1;
  else if (value > 1) value = 1;
  return Math.acos(value);
}
