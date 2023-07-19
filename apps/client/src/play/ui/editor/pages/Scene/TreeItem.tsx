import { useSceneStore } from "@unavi/react-client";
import { IoMdExpand } from "react-icons/io";

interface Props {
  id: bigint;
}

export default function TreeItem({ id }: Props) {
  const name = useSceneStore((state) => state.items.get(id)?.name);
  const selectedId = useSceneStore((state) => state.selectedId);

  function select() {
    useSceneStore.setState({ selectedId: id });
  }

  function expand() {
    useSceneStore.setState({ sceneTreeId: id, selectedId: id });
  }

  const isSelected = selectedId === id;

  return (
    <div className="group relative flex space-x-1">
      <button
        onClick={select}
        className={`w-full rounded px-2 py-0.5 text-start active:opacity-90 ${
          isSelected
            ? "bg-white/10 group-hover:bg-white/20"
            : "group-hover:bg-white/10"
        }`}
      >
        {name || `(${id.toString()})`}
      </button>

      <button
        onClick={expand}
        className="absolute inset-y-0 right-0 hidden w-8 items-center justify-center rounded text-lg hover:opacity-70 active:opacity-60 group-hover:flex"
      >
        <IoMdExpand />
      </button>
    </div>
  );
}
