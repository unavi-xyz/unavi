import useLocalWorld from "../../helpers/localWorlds/useLocalWorld";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const world = useLocalWorld(id);

  return (
    <div
      className="w-64 h-48 bg-neutral-200 rounded hover:cursor-pointer
                 ring-1 ring-black flex flex-col"
    >
      <div className="h-36">
        {world?.image && (
          <img
            src={world?.image}
            alt="scene preview"
            className="w-full h-full object-cover rounded-t"
          />
        )}
      </div>

      <hr className="border-black" />

      <div className="p-3">{world?.name}</div>
    </div>
  );
}
