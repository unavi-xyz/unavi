import { MdClose } from "react-icons/md";

import IconButton from "../../ui/IconButton";
import { useScript } from "../hooks/useScript";
import { useEditorStore } from "../store";

interface Props {
  scriptId: string;
}

export default function ScriptMenu({ scriptId }: Props) {
  const script = useScript(scriptId);

  if (!script) return null;

  return (
    <div>
      <div className="flex h-9 items-center justify-between px-4">
        <div className="text-lg">{script.name}</div>

        <IconButton
          cursor="pointer"
          onClick={() => useEditorStore.setState({ openScriptId: null })}
        >
          <MdClose />
        </IconButton>
      </div>
    </div>
  );
}
