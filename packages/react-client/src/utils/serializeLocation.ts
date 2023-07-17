import { LOCATION_ROUNDING } from "../constants";

const LOCATION_BYTES = 21;

/**
 * Serialize a location into a buffer.
 *
 * Buffer is 21 bytes long.
 * 1 byte for player id (Uint8)
 * 3 * 4 bytes for position (Int32)
 * 4 * 2 bytes for rotation (Int16)
 */
export function serializeLocation(
  playerId: number,
  posX: number,
  posY: number,
  posZ: number,
  rotX: number,
  rotY: number,
  rotZ: number,
  rotW: number
) {
  const buffer = new ArrayBuffer(LOCATION_BYTES);
  const view = new DataView(buffer);

  view.setUint8(0, playerId);

  view.setInt32(1, posX * LOCATION_ROUNDING.POSITION);
  view.setInt32(5, posY * LOCATION_ROUNDING.POSITION);
  view.setInt32(9, posZ * LOCATION_ROUNDING.POSITION);

  view.setInt16(13, rotX * LOCATION_ROUNDING.ROTATION);
  view.setInt16(15, rotY * LOCATION_ROUNDING.ROTATION);
  view.setInt16(17, rotZ * LOCATION_ROUNDING.ROTATION);
  view.setInt16(19, rotW * LOCATION_ROUNDING.ROTATION);

  return buffer;
}
