const colors = {
  blue: "#3b82f6",
  cyan: "#22d3ee",
  green: "#22c55e",
  light: "#a3a3a3",
  lime: "#84cc16",
  neutral: "#262626",
  purple: "#a855f7",
  red: "#ef4444",
  sky: "#0ea5e9",
} as const;

export const valueColorsMap: Record<string, string> = {
  boolean: colors.red,
  euler: colors.sky,
  float: colors.green,
  flow: colors.neutral,
  integer: colors.lime,
  mat3: colors.light,
  mat4: colors.light,
  number: colors.green,
  quat: colors.blue,
  string: colors.purple,
  vec2: colors.cyan,
  vec3: colors.sky,
  vec4: colors.blue,
};
