import useLocalWorld from "../../helpers/localWorlds/useLocalWorld";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const world = useLocalWorld(id);

  return (
    <div
      className="relative w-64 h-48 bg-neutral-200 rounded-md hover:cursor-pointer
                 flex flex-col shadow hover:-translate-y-2 transition-all"
    >
      <div className="h-full">
        {world?.image && (
          <img
            src={world?.image}
            alt="scene preview"
            className="w-full h-full object-cover rounded-md opacity-100"
          />
        )}
      </div>

      <div
        className="absolute bottom-0 px-3 py-2 text-lg rounded-b-md w-full bg-white
                   bg-opacity-60 bg-blur-filter"
      >
        {world?.name}
      </div>
    </div>
  );
}
