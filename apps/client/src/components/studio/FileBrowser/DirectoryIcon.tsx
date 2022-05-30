import { VscFolder } from "react-icons/vsc";

interface Props {
  handle: FileSystemDirectoryHandle;
}

export default function DirectoryIcon({ handle }: Props) {
  return (
    <div
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
