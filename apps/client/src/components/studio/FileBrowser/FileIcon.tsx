import {
  VscCircleLargeOutline,
  VscFile,
  VscFileBinary,
  VscFileCode,
} from "react-icons/vsc";

import FileImageIcon from "./FileImageIcon";

const imageExtensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"];
const codeExtensions = ["js", "ts"];
const modelExtensions = ["gltf", "glb", "fbx", "obj"];
const materialExtension = ["material"];
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
      className="group aspect-square h-28 p-4 rounded-xl
                 flex flex-col items-center justify-between cursor-default select-none"
    >
      <div>
        {imageExtensions.includes(fileExtension) ? (
          <FileImageIcon handle={handle} />
        ) : codeExtensions.includes(fileExtension) ? (
          <VscFileCode className="text-5xl" />
        ) : modelExtensions.includes(fileExtension) ? (
          <VscFileBinary className="text-5xl" />
        ) : materialExtension.includes(fileExtension) ? (
          <VscCircleLargeOutline className="text-5xl" />
        ) : (
          <VscFile className="text-5xl" />
        )}
      </div>

      <div className="w-24 flex justify-center text-sm font-bold group-hover:break-all">
        <div className="text-ellipsis overflow-clip whitespace-nowrap group-hover:whitespace-normal">
          {fileName}
        </div>
      </div>
    </div>
  );
}
