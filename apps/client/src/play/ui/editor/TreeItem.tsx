import { useSceneStore } from "@unavi/react-client";

interface Props {
  id: bigint;
}

export default function TreeItem({ id }: Props) {
  const name = useSceneStore((state) => state.items.get(id)?.name);
  const childrenIds = useSceneStore(
    (state) => state.items.get(id)?.childrenIds
  );
  const selectedId = useSceneStore((state) => state.selectedId);

  function select() {
    useSceneStore.setState({ selectedId: id });
  }

  const isSelected = selectedId === id;

  return (
    <div>
      <button
        onClick={select}
        className={`w-full rounded px-2 py-0.5 text-start active:opacity-90 ${
          isSelected ? "bg-white/10 hover:bg-white/20" : "hover:bg-white/10"
        }`}
      >
        {name || `(${id.toString()})`}
      </button>

      <div className="">
        {childrenIds?.map((childId) => (
          <TreeItem key={childId.toString()} id={childId} />
        ))}
      </div>
    </div>
  );
}
