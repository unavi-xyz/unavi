import {
  VscFile,
  VscFileBinary,
  VscFileCode,
  VscFileMedia,
} from "react-icons/vsc";

const imageExtensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"];
const codeExtensions = ["js", "ts"];
const modelExtensions = ["gltf", "glb", "fbx", "obj"];
const hiddenExtensions = ["crswap"];

interface Props {
  handle: FileSystemFileHandle;
}

export default function FileIcon({ handle }: Props) {
  const fileExtension = handle.name.split(".").pop() ?? "";
  const fileName = handle.name.split(".").slice(0, -1).join(".");

  if (hiddenExtensions.includes(fileExtension)) return null;

  return (
    <div
      className="group aspect-square h-28 p-4 rounded-xl hover:ring-1 hover:ring-outline
                 flex flex-col items-center justify-between cursor-default select-none"
    >
      <div className="text-5xl">
        {imageExtensions.includes(fileExtension) ? (
          <VscFileMedia />
        ) : codeExtensions.includes(fileExtension) ? (
          <VscFileCode />
        ) : modelExtensions.includes(fileExtension) ? (
          <VscFileBinary />
        ) : (
          <VscFile />
        )}
      </div>

      <div className="w-24 flex justify-center text-sm group-hover:break-all">
        <div className="text-ellipsis overflow-clip whitespace-nowrap group-hover:whitespace-normal">
          {fileName}
        </div>
      </div>
    </div>
  );
}
