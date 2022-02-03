export function shortenDid(did: string) {
  const chars = 4;
  return `${did.slice(0, chars + 6)}...${did.slice(
    did.length - chars,
    did.length
  )}`;
}
