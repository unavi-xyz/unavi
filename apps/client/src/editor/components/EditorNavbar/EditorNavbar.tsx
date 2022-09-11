import { useRouter } from "next/router";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { HiCubeTransparent } from "react-icons/hi";
import {
  MdArrowBackIosNew,
  MdOutlineGridOn,
  MdPreview,
  MdSync,
} from "react-icons/md";

import IconButton from "../../../ui/base/IconButton";
import Tooltip from "../../../ui/base/Tooltip";
import { useSave } from "../../hooks/useSave";
import { useEditorStore } from "../../store";
import ToolButton from "./ToolButton";

export default function EditorNavbar() {
  const router = useRouter();
  const id = router.query.id;

  const debug = useEditorStore((state) => state.debug);
  const grid = useEditorStore((state) => state.grid);
  const name = useEditorStore((state) => state.name);

  const { save } = useSave();

  function handleToggleDebug() {
    useEditorStore.setState({ debug: !debug, selectedId: null });
  }

  function handleToggleGrid() {
    useEditorStore.setState({ grid: !grid });
  }

  async function handleBack() {
    await save();
    router.push(`/project/${id}`);
  }

  async function handlePreview() {
    await save();
    router.push(`/editor/${id}/preview`);
  }

  return (
    <div className="flex justify-between items-center h-full px-4 py-2">
      <div className="w-full flex items-center space-x-4 text-lg">
        <div
          onClick={handleBack}
          className="cursor-pointer transition text-outline hover:text-inherit p-1"
        >
          <MdArrowBackIosNew />
        </div>

        <div>{name}</div>
      </div>

      <div className="w-full h-full flex justify-center items-center space-x-2">
        <ToolButton tool="translate">
          <BiMove />
        </ToolButton>
        <ToolButton tool="rotate">
          <MdSync />
        </ToolButton>
        <ToolButton tool="scale">
          <CgArrowsExpandUpRight />
        </ToolButton>
      </div>

      <div className="w-full h-full flex justify-end items-center space-x-2">
        <div className="h-full aspect-square">
          <Tooltip text={`${grid ? "Hide" : "Show"} Grid`} placement="bottom">
            <IconButton selected={grid} onClick={handleToggleGrid}>
              <MdOutlineGridOn />
            </IconButton>
          </Tooltip>
        </div>

        <div className="h-full aspect-square">
          <Tooltip
            text={`${debug ? "Hide" : "Show"} Colliders`}
            placement="bottom"
          >
            <IconButton selected={debug} onClick={handleToggleDebug}>
              <HiCubeTransparent />
            </IconButton>
          </Tooltip>
        </div>

        <div className="h-full aspect-square">
          <Tooltip text="Preview" placement="bottom">
            <div className="h-full">
              <IconButton onClick={handlePreview}>
                <MdPreview />
              </IconButton>
            </div>
          </Tooltip>
        </div>

        {/* <div className="pl-2">{handle ? <PublishButton /> : <LoginButton />}</div> */}
      </div>
    </div>
  );
}
