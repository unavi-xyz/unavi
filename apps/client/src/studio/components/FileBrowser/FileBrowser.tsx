import { useEffect, useState } from "react";

import {
  compareDirectoryArrays,
  compareFileArrays,
  readDirectoryContents,
} from "../../../studio/filesystem";
import { useStudioStore } from "../../../studio/store";
import DirectoryIcon from "./DirectoryIcon";
import DirectoryRow from "./DirectoryRow";
import FileIcon from "./FileIcon";

export default function FileBrowser() {
  const rootHandle = useStudioStore((state) => state.rootHandle);
  const selectedDirectory = useStudioStore((state) => state.selectedDirectory);

  const [filesHandles, setFilesHandles] = useState<FileSystemFileHandle[]>([]);
  const [directoryHandles, setDirectoryHandles] = useState<FileSystemDirectoryHandle[]>([]);

  useEffect(() => {
    async function readEntries() {
      if (!rootHandle) return;

      const directoryHandle = selectedDirectory ?? rootHandle;
      const { files, directories } = await readDirectoryContents(directoryHandle);

      const fileDiff = compareFileArrays(filesHandles, files);
      const directoryDiff = compareDirectoryArrays(directoryHandles, directories);

      if (fileDiff.length > 0) setFilesHandles(files);
      if (directoryDiff.length > 0) setDirectoryHandles(directories);
    }

    readEntries();

    const interval = setInterval(readEntries, 2000);

    return () => {
      clearInterval(interval);
    };
  }, [directoryHandles, filesHandles, rootHandle, selectedDirectory]);

  if (!rootHandle) return null;

  return (
    <div className="flex h-full">
      <div className="w-64 p-4">
        <DirectoryRow handle={rootHandle} defaultExpanded />
      </div>

      <div className="flex flex-wrap space-x-2 p-4">
        {directoryHandles.map((handle) => {
          return <DirectoryIcon key={handle.name} handle={handle} />;
        })}

        {filesHandles.map((handle) => {
          return <FileIcon key={handle.name} handle={handle} />;
        })}
      </div>
    </div>
  );
}
