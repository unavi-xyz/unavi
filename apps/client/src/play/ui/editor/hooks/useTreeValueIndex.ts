import { TreeItem, useSceneStore } from "@unavi/engine";

type ArrayKey<T extends keyof TreeItem> = Exclude<
  T,
  {
    [K in T]: TreeItem[K] extends Array<any> | Record<string, any> ? never : K;
  }[T]
>;

type Keys = ArrayKey<keyof TreeItem>;

export function useTreeValueIndex<T extends Keys, U extends keyof TreeItem[T]>(
  entityId: bigint | undefined,
  key: T,
  index: U
): TreeItem[T][U] | undefined {
  const value = useSceneStore((state) =>
    entityId ? state.items.get(entityId)?.[key]?.[index] : undefined
  );
  return value;
}
