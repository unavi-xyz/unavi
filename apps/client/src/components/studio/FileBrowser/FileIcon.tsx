import {
  VscFile,
  VscFileBinary,
  VscFileCode,
  VscFileMedia,
} from "react-icons/vsc";

const imageExtensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"];
const codeExtensions = ["js", "ts"];
const modelExtensions = ["gltf", "glb", "fbx", "obj"];

interface Props {
  handle: FileSystemFileHandle;
}

export default function FileIcon({ handle }: Props) {
  const fileExtension = handle.name.split(".").pop() ?? "";

  return (
    <div
      className="aspect-square h-28 p-4 rounded-xl hover:ring-1 hover:ring-outline
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

      <div className="break-all text-sm">{handle.name}</div>
    </div>
  );
}
