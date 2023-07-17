/**
 * Parses a handle into a username and home server
 *
 * @example
 * parseHandle("@alice:unavi.xyz") // { home: "unavi.xyz", username: "alice" }
 */
export function parseHandle(handle: string) {
  const [front, home] = handle.split(":");
  const username = front?.slice(1);
  return { home, username };
}
