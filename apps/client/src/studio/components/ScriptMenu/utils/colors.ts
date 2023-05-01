const colors = {
  blue: "#3b82f6",
  cyan: "#22d3ee",
  green: "#22c55e",
  lime: "#84cc16",
  neutral: "#262626",
  light: "#a3a3a3",
  purple: "#a855f7",
  red: "#ef4444",
  sky: "#0ea5e9",
} as const;

export const valueColorsMap: Record<string, string> = {
  flow: colors.neutral,
  number: colors.green,
  float: colors.green,
  integer: colors.lime,
  boolean: colors.red,
  string: colors.purple,
  vec2: colors.cyan,
  vec3: colors.sky,
  euler: colors.sky,
  vec4: colors.blue,
  quat: colors.blue,
  mat3: colors.light,
  mat4: colors.light,
};
