import { Scene } from "@wired-xr/engine";

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
  await writableStream.write(JSON.stringify(newProject, null, 2));
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

export async function findFilePath(
  directoryHandle: FileSystemDirectoryHandle,
  fileHandle: FileSystemFileHandle
): Promise<string | undefined> {
  const dirName = await directoryHandle.name;

  for await (const child of directoryHandle.values()) {
    //test if same name
    if (child.kind === "file" && child.name === fileHandle.name) {
      //test if same content
      const file = await fileHandle.getFile();
      const fileContents = await file.text();

      const file2 = await child.getFile();
      const fileContents2 = await file2.text();

      if (fileContents === fileContents2) {
        return `${dirName}/${fileHandle.name}`;
      }
    }
  }

  //file was not found, search children
  for await (const child of directoryHandle.values()) {
    if (child.kind === "directory") {
      const foundPath = await findFilePath(child, fileHandle);
      if (foundPath) return `${dirName}/${foundPath}`;
    }
  }
}

export async function getFileByPath(
  path: string,
  directoryHandle: FileSystemDirectoryHandle
): Promise<FileSystemFileHandle | undefined> {
  try {
    const splitPath = path
      .split("/")
      .filter((p) => p !== "")
      .splice(1);

    if (splitPath.length === 1) {
      const handle = await directoryHandle.getFileHandle(splitPath[0]);
      return handle;
    }

    const handle = await directoryHandle.getDirectoryHandle(splitPath[0]);
    const newPath = splitPath.join("/");
    return await getFileByPath(newPath, handle);
  } catch (err) {
    console.error(err);
    return;
  }
}

export function compareFileArrays(
  a: FileSystemFileHandle[],
  b: FileSystemFileHandle[]
) {
  const difference = a
    .filter((aHandle) => {
      return !b.some((bHandle) => bHandle.name === aHandle.name);
    })
    .concat(
      b.filter((bHandle) => {
        return !a.some((aHandle) => aHandle.name === bHandle.name);
      })
    );

  return difference;
}

export function compareDirectoryArrays(
  a: FileSystemDirectoryHandle[],
  b: FileSystemDirectoryHandle[]
) {
  const difference = a
    .filter((aHandle) => {
      return !b.some((bHandle) => bHandle.name === aHandle.name);
    })
    .concat(
      b.filter((bHandle) => {
        return !a.some((aHandle) => aHandle.name === bHandle.name);
      })
    );

  return difference;
}
