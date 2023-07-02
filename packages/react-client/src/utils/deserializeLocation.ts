import { LOCATION_ROUNDING } from "../constants";

/**
 * Deserialize a location from a buffer.
 *
 * Array is 8 items long.
 * Index 0 is player id
 * Index 1-3 is position
 * Index 4-7 is rotation
 */
export function deserializeLocation(buffer: ArrayBuffer) {
  const view = new DataView(buffer);

  const playerId = view.getUint8(0);

  const posX = view.getInt32(1) / LOCATION_ROUNDING.POSITION;
  const posY = view.getInt32(5) / LOCATION_ROUNDING.POSITION;
  const posZ = view.getInt32(9) / LOCATION_ROUNDING.POSITION;

  const rotX = view.getInt16(13) / LOCATION_ROUNDING.ROTATION;
  const rotY = view.getInt16(15) / LOCATION_ROUNDING.ROTATION;
  const rotZ = view.getInt16(17) / LOCATION_ROUNDING.ROTATION;
  const rotW = view.getInt16(19) / LOCATION_ROUNDING.ROTATION;

  return [playerId, posX, posY, posZ, rotX, rotY, rotZ, rotW];
}
