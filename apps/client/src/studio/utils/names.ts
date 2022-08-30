import { useStudioStore } from "../store";

export function addObjectName(name: string): number {
  const names = useStudioStore.getState().names;
  const id = getId(names);
  names[id] = name;
  useStudioStore.setState({ names });
  return id;
}

export function removeObjectName(id: number) {
  const names = useStudioStore.getState().names;
  delete names[id];
  useStudioStore.setState({ names });
}

function getId(names: { [id: number]: string }) {
  let id = 0;
  while (names[id]) id++;
  return id;
}
