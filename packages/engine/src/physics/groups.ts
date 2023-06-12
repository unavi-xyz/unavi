export enum CollisionGroup {
  None = 0,
  Static = 1,
  Dynamic = 2,
  Player = 4,
  All = 65535,
}

/**
 * @param membership The groups the collider is part of (should be a valid UInt16)
 * @param filter The groups the collider can interact with (should be a valid UInt16)
 */
function constructCollisionGroups(membership: number, filter: number): number {
  return (membership << 16) + filter;
}

export const COLLISION_GROUP = {
  player: constructCollisionGroups(
    CollisionGroup.Player,
    CollisionGroup.Static | CollisionGroup.Dynamic
  ),

  static: constructCollisionGroups(
    CollisionGroup.Static,
    CollisionGroup.Dynamic | CollisionGroup.Player
  ),
} as const;
