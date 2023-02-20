import { BehaviorNode, BehaviorNodeExtras } from "engine";

import { useNode } from "../../hooks/useNode";
import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { useEditorStore } from "../../store";
import TextInput from "../ui/TextInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  nodeId: string;
  scriptId: string;
}

export default function ScriptComponent({ nodeId, scriptId }: Props) {
  const node = useNode(nodeId);
  const extras = useNodeAttribute(nodeId, "extras");

  if (!node || !extras) return null;

  const scripts = extras.scripts ?? [];
  const script = scripts.find((script) => script.id === scriptId);

  if (!script) return null;

  return (
    <ComponentMenu
      title="Script"
      onRemove={() => {
        if (!node) return;

        const { openScriptId, engine } = useEditorStore.getState();
        if (openScriptId === scriptId) useEditorStore.setState({ openScriptId: null });
        if (!engine) return;

        // Remove script from node extras
        const newExtras = { ...extras };
        newExtras.scripts = newExtras.scripts?.filter((script) => script.id !== scriptId);
        node.setExtras(newExtras);

        // Remove behavior nodes
        engine.scene.extensions.behavior.listProperties().forEach((property) => {
          if (!(property instanceof BehaviorNode)) return;
          const extras = property.getExtras() as BehaviorNodeExtras;
          if (extras.script === scriptId) property.dispose();
        });
      }}
    >
      <MenuRows titles={["Name"]}>
        <TextInput
          value={script.name}
          onChange={(e) => {
            const newExtras = { ...extras };

            newExtras.scripts = newExtras.scripts?.map((script) => {
              if (script.id === scriptId) {
                return { ...script, name: e.target.value };
              }

              return script;
            });

            node.setExtras(newExtras);
          }}
        />
      </MenuRows>

      <div className="flex justify-end">
        <button
          onClick={() => useEditorStore.setState({ openScriptId: scriptId })}
          className="rounded-full bg-neutral-200/80 px-8 py-0.5 transition hover:bg-neutral-300/80 active:opacity-80"
        >
          Open
        </button>
      </div>
    </ComponentMenu>
  );
}
