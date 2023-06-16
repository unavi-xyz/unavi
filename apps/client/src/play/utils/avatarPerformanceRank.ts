import { ModelStats } from "./getModelStats";

export type AvatarPerformanceRank =
  | "Excellent"
  | "Good"
  | "Medium"
  | "Poor"
  | "Very Poor";

type WithoutVeryPoor = Exclude<AvatarPerformanceRank, "Very Poor">;
type AvatarStats = Omit<ModelStats, "fileSize">;

const thresholds: Record<WithoutVeryPoor, AvatarStats> = {
  Excellent: {
    boneCount: 75,
    materialCount: 1,
    meshCount: 1,
    skinCount: 1,
    triangleCount: 7_500,
  },

  Good: {
    boneCount: 90,
    materialCount: 1,
    meshCount: 1,
    skinCount: 1,
    triangleCount: 10_000,
  },

  Medium: {
    boneCount: 150,
    materialCount: 2,
    meshCount: 2,
    skinCount: 2,
    triangleCount: 15_000,
  },

  Poor: {
    boneCount: 150,
    materialCount: 4,
    meshCount: 2,
    skinCount: 2,
    triangleCount: 20_000,
  },
};

export function avatarPerformanceRank(
  stats: ModelStats
): AvatarPerformanceRank {
  const keys = Object.keys(thresholds) as WithoutVeryPoor[];

  for (let i = 0; i < keys.length; i++) {
    const key = keys[i];
    if (!key) throw new Error("Invalid key");

    const threshold = thresholds[key];

    if (
      stats.triangleCount <= threshold.triangleCount &&
      stats.materialCount <= threshold.materialCount &&
      stats.meshCount <= threshold.meshCount &&
      stats.skinCount <= threshold.skinCount &&
      stats.boneCount <= threshold.boneCount
    ) {
      return key;
    }
  }

  return "Very Poor";
}
