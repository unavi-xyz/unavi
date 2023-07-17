/**
 * There is currently an issue in drizzle types when querying relations using `with`
 * This type will ensure a key is marked as possibly being undefined
 */
export type FixWith<T extends Record<string, any>, U extends keyof T> = Omit<
  T,
  U
> & {
  [K in U]: T[K] | undefined;
};
