import { IoIosMove } from "react-icons/io";
import { BsArrowRepeat, BsArrowsAngleExpand } from "react-icons/bs";
import { useAtom } from "jotai";

import { Tool } from "../../../helpers/editor/types";
import { toolAtom } from "../../../helpers/editor/state";

export default function MiddleButtons() {
  const [tool, setTool] = useAtom(toolAtom);

  return (
    <div className="flex items-center space-x-2">
      <MiddleButton
        selected={tool === Tool.translate}
        onClick={() => setTool(Tool.translate)}
      >
        <IoIosMove />
      </MiddleButton>
      <MiddleButton
        selected={tool === Tool.rotate}
        onClick={() => setTool(Tool.rotate)}
      >
        <BsArrowRepeat />
      </MiddleButton>
      <MiddleButton
        selected={tool === Tool.scale}
        onClick={() => setTool(Tool.scale)}
      >
        <BsArrowsAngleExpand className="text-[1.4rem]" />
      </MiddleButton>
    </div>
  );
}

function MiddleButton({ selected, children, ...rest }) {
  const css = selected ? "bg-neutral-100" : null;

  return (
    <div
      {...rest}
      className={`hover:bg-neutral-100 rounded-md w-10 h-10 hover:cursor-pointer
                  flex items-center justify-center ${css}`}
    >
      {children}
    </div>
  );
}
