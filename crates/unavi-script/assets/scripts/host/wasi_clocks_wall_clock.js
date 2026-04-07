export function now() {
  const ms = BigInt(Date.now());
  return { seconds: ms / 1000n, nanoseconds: Number((ms % 1000n) * 1000000n) };
}
export function resolution() {
  return { seconds: 0n, nanoseconds: 1000000 };
}
