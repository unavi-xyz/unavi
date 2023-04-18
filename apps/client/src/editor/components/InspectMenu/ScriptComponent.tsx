import { Node } from "@gltf-transform/core";
import { BehaviorNode, BehaviorNodeExtras } from "@unavi/gltf-extensions";

import { useNodeExtras } from "../../hooks/useNodeExtras";
import { useEditor } from "../Editor";
import EditorInput from "../ui/EditorInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  node: Node;
  scriptId: string;
}

export default function ScriptComponent({ node, scriptId }: Props) {
  const { engine, scriptId: openScriptId, setScriptId } = useEditor();
  const extras = useNodeExtras(node);

  if (!node || !extras) return null;

  const scripts = extras.scripts ?? [];
  const script = scripts.find((script) => script.id === scriptId);

  if (!script) return null;

  return (
    <ComponentMenu
      title="Script"
      onRemove={() => {
        if (!node) return;

        if (openScriptId === scriptId) setScriptId(null);
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

        // Remove unused variables
        engine.scene.extensions.behavior.listVariables().forEach((variable) => {
          if (variable.listParents().length === 0) variable.dispose();
        });
      }}
    >
      <MenuRows titles={["Name"]}>
        <EditorInput
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
          onClick={() => setScriptId(scriptId)}
          className="rounded-full bg-neutral-200/80 px-8 py-0.5 transition hover:bg-neutral-300/80 active:opacity-80"
        >
          Open
        </button>
      </div>
    </ComponentMenu>
  );
}
