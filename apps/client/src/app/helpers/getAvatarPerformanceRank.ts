import { AvatarStats } from "../../server/helpers/getAvatarStats";

export type AvatarPerformanceRank =
  | "Excellent"
  | "Good"
  | "Medium"
  | "Poor"
  | "Very Poor";

type WithoutVeryPoor = Exclude<AvatarPerformanceRank, "Very Poor">;

const thresholds: Record<WithoutVeryPoor, AvatarStats> = {
  Excellent: {
    polygonCount: 7_500,
    materialCount: 1,
    meshCount: 1,
    skinCount: 1,
    boneCount: 75,
  },

  Good: {
    polygonCount: 10_000,
    materialCount: 1,
    meshCount: 1,
    skinCount: 1,
    boneCount: 90,
  },

  Medium: {
    polygonCount: 15_000,
    materialCount: 2,
    meshCount: 2,
    skinCount: 2,
    boneCount: 150,
  },

  Poor: {
    polygonCount: 20_000,
    materialCount: 4,
    meshCount: 2,
    skinCount: 2,
    boneCount: 150,
  },
};

export function getAvatarPerformanceRank(
  stats: AvatarStats
): AvatarPerformanceRank {
  const keys = Object.keys(thresholds) as WithoutVeryPoor[];

  for (let i = 0; i < keys.length; i++) {
    const key = keys[i];
    if (!key) throw new Error("Invalid key");

    const threshold = thresholds[key];

    if (
      stats.polygonCount <= threshold.polygonCount &&
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
