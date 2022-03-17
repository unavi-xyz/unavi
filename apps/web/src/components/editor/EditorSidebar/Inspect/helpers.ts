import { useStore } from "../../../../helpers/editor/store";

export function handleChange(value: any, key: string) {
  const changes = { [key]: value };
  const id = useStore.getState().selected.id;
  useStore.getState().updateInstanceParams(id, changes);
}

export function getHandleChange(key: string) {
  return (value: any) => handleChange(value, key);
}
