import { IoIosMove } from "react-icons/io";
import { BsArrowRepeat, BsArrowsAngleExpand } from "react-icons/bs";

import { Tooltip } from "../../base";
import { editorManager, useStore } from "../helpers/store";

export default function MiddleButtons() {
  const tool = useStore((state) => state.tool);

  return (
    <div className="flex items-center space-x-2">
      <Tooltip text="Translate (W)" placement="bottom">
        <MiddleButton
          selected={tool === "translate"}
          onClick={() => editorManager.setTool("translate")}
        >
          <IoIosMove />
        </MiddleButton>
      </Tooltip>
      <Tooltip text="Rotate (E)" placement="bottom">
        <MiddleButton
          selected={tool === "rotate"}
          onClick={() => editorManager.setTool("rotate")}
        >
          <BsArrowRepeat />
        </MiddleButton>
      </Tooltip>
      <Tooltip text="Scale (R)" placement="bottom">
        <MiddleButton
          selected={tool === "scale"}
          onClick={() => editorManager.setTool("scale")}
        >
          <BsArrowsAngleExpand className="text-[1.4rem]" />
        </MiddleButton>
      </Tooltip>
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
