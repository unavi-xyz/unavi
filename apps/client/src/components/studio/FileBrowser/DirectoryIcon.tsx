import { VscFolder } from "react-icons/vsc";

import { useStudioStore } from "../../../helpers/studio/store";

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

      <div className="break-all text-sm">{handle.name}</div>
    </div>
  );
}
