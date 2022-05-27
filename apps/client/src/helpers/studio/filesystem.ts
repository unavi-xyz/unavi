import { Scene } from "@wired-xr/scene";

import { PROJECT_FILE_NAME } from "./constants";
import { useStudioStore } from "./store";
import { Project } from "./types";

export async function readProject() {
  const root = useStudioStore.getState().rootHandle;
  if (!root) return;

  const handle = await root.getFileHandle(PROJECT_FILE_NAME);
  const file = await handle.getFile();
  const json = await file.text();

  const project: Project = JSON.parse(json);
  return project;
}

export async function writeScene(scene: Scene) {
  const root = useStudioStore.getState().rootHandle;
  if (!root) return;

  const project = await readProject();
  if (!project) return;

  const newProject: Project = {
    ...project,
    scene,
  };

  const handle = await root.getFileHandle(PROJECT_FILE_NAME);
  const writableStream = await handle.createWritable();
  await writableStream.write(JSON.stringify(newProject));
  await writableStream.close();
}

export async function readDirectoryContents(
  directoryHandle: FileSystemDirectoryHandle
) {
  const files: FileSystemFileHandle[] = [];
  const directories: FileSystemDirectoryHandle[] = [];

  for await (const handle of directoryHandle.values()) {
    if (handle.kind === "file") {
      files.push(handle);
    } else {
      directories.push(handle);
    }
  }

  return { files, directories };
}

export async function findFilePathByName(
  directoryHandle: FileSystemDirectoryHandle,
  name: string
): Promise<string | undefined> {
  for await (const handle of directoryHandle.values()) {
    if (handle.kind === "file" && handle.name === name) {
      return `/${name}`;
    }
  }

  //file was not found, search children
  for await (const handle of directoryHandle.values()) {
    if (handle.kind === "directory") {
      const foundPath = await findFilePathByName(handle, name);
      if (foundPath) return `/${foundPath}`;
    }
  }
}

export async function readFileByPath(path: string) {
  const root = useStudioStore.getState().rootHandle;
  if (!root) return;

  const splitPath = path.split("/").filter((p) => p !== "");

  const handle = await root.getFileHandle(splitPath[0]);
  const file = await handle.getFile();

  return file;
}
