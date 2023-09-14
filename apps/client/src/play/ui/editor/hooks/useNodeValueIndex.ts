import { NodeItem, useSceneStore } from "@unavi/engine";

type ArrayKey<T extends keyof NodeItem> = Exclude<
  T,
  {
    [K in T]: NodeItem[K] extends Array<any> | Record<string, any> ? never : K;
  }[T]
>;

type Keys = ArrayKey<keyof NodeItem>;

export function useNodeValueIndex<T extends Keys, U extends keyof NodeItem[T]>(
  entityId: bigint | undefined,
  key: T,
  index: U
): NodeItem[T][U] | undefined {
  const value = useSceneStore((state) =>
    entityId ? state.items.get(entityId)?.[key]?.[index] : undefined
  );
  return value;
}
