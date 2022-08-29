import { useStudioStore } from "../store";

export function setObjectName(eiud: number, name: string) {
  const names = useStudioStore.getState().names;
  names[eiud] = name;
  useStudioStore.setState({ names });
}
