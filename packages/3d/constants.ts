//centers clicks to the middle of the screen
export const RAYCASTER_SETTINGS = {
  computeOffsets: (e: any) => ({
    offsetX: e.target.width / 2,
    offsetY: e.target.height / 2,
  }),
};

export enum PHYSICS_GROUPS {
  NONE = 0,
  PLAYER = 1,
  WORLD = 2,
  OBJECTS = 4,
}
