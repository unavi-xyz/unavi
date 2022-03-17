import { useStore } from "../../../../helpers/editor/store";

export function handleChange(value: any, key: string) {
  const changes = { [key]: value };
  const id = useStore.getState().selected.id;
  useStore.getState().updateInstanceParams(id, changes);
}

export function getHandleChange(key: string) {
  return (value: any) => handleChange(value, key);
}

export function readFile(file: File) {
  return new Promise<ArrayBuffer>((resolve, reject) => {
    // Create file reader
    let reader = new FileReader();

    // Register event listeners
    reader.addEventListener("loadend", (e) =>
      resolve(e.target.result as ArrayBuffer)
    );
    reader.addEventListener("error", reject);

    // Read file
    reader.readAsArrayBuffer(file);
  });
}

export function readFileAsDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    // Create file reader
    let reader = new FileReader();

    // Register event listeners
    reader.addEventListener("loadend", (e) =>
      resolve(e.target.result as string)
    );
    reader.addEventListener("error", reject);

    // Read file
    reader.readAsDataURL(file);
  });
}
