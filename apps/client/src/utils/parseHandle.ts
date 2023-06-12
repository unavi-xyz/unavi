/**
 * Parses a handle into a username and domain
 *
 * @example
 * parseHandle("alice@unavi.xyz") // { username: "alice", domain: "unavi.xyz" }
 */
export function parseHandle(handle: string) {
  const [username, domain] = handle.split("@");
  return { domain, username };
}
