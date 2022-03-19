import { AssetName, ASSETS, Params } from "3d";
import { useStore } from "../../helpers/store";

export function handleChange(value: any, key: string) {
  const changes = { [key]: value };
  const id = useStore.getState().selected.id;
  useStore.getState().updateInstanceParams(id, changes);
}

export function getHandleChange(key: string) {
  return (value: any) => handleChange(value, key);
}

export function useSections(name: AssetName) {
  if (!name) return;
  const sections = Object.keys(ASSETS[name].params) as Array<keyof Params>;
  return sections;
}
