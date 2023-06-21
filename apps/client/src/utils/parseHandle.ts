/**
 * Parses a handle into a username and home server
 *
 * @example
 * parseHandle("alice@unavi.xyz") // { username: "alice", home: "unavi.xyz" }
 */
export function parseHandle(handle: string) {
  const [username, home] = handle.split("@");
  return { home, username };
}
