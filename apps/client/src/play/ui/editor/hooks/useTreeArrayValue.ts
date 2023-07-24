import { TreeItem, useSceneStore } from "@unavi/engine";

type ArrayKey<T extends keyof TreeItem> = Exclude<
  T,
  {
    [K in T]: TreeItem[K] extends Array<any> ? never : K;
  }[T]
>;

type Keys = ArrayKey<keyof TreeItem>;

export function useTreeArrayValue<T extends Keys, U extends keyof TreeItem[T]>(
  id: bigint | undefined,
  key: T,
  index: U
): TreeItem[T][U] | undefined {
  const value = useSceneStore((state) =>
    id ? state.items.get(id)?.[key][index] : undefined
  );

  return value;
}
