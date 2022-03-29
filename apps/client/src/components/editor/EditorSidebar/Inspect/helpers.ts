import { sceneManager, useStore } from "../../helpers/store";

export function handleChange(value: any, key: string) {
  const changes = { [key]: value };
  const id = useStore.getState().selected?.id;
  if (!id) return;
  sceneManager.editInstance(id, changes);
}

export function getHandleChange(key: string) {
  return (value: any) => handleChange(value, key);
}
