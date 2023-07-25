export function splitHandle(handle: string) {
  const parts = handle.split(":");
  const username = parts[0]?.slice(1);
  const server = parts[1];

  if (!username || !server) {
    return { server: null, username: null };
  }

  return { server, username };
}
