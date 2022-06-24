import { VscFolder } from "react-icons/vsc";

import { useStudioStore } from "../../../studio/store";

interface Props {
  handle: FileSystemDirectoryHandle;
}

export default function DirectoryIcon({ handle }: Props) {
  function handleDoubleClick() {
    useStudioStore.setState({ selectedDirectory: handle });
  }

  return (
    <div
      onDoubleClick={handleDoubleClick}
      className="aspect-square h-28 p-4 rounded-xl hover:ring-1 hover:ring-outline
                 flex flex-col items-center justify-between cursor-default select-none"
    >
      <div className="text-5xl">
        <VscFolder />
      </div>

      <div className="w-24 flex justify-center text-sm group-hover:break-all">
        <div className="text-ellipsis overflow-clip whitespace-nowrap group-hover:whitespace-normal">
          {handle.name}
        </div>
      </div>
    </div>
  );
}
