import { useLocalWorld } from "../../helpers/localWorlds/useLocalWorld";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const world = useLocalWorld(id);

  return (
    <div className="w-full h-40">
      <div
        className="relative w-full h-full rounded-xl hover:cursor-pointer
                 flex flex-col"
      >
        <div className="h-full">
          {world?.image && (
            <img
              src={world.image}
              alt="scene image"
              className="w-full h-full object-cover rounded-xl opacity-100"
            />
          )}
        </div>

        {world?.name && (
          <div className="absolute bottom-0 px-3 py-2 text-lg rounded-b-xl w-full bg-neutral-100">
            {world.name}
          </div>
        )}
      </div>
    </div>
  );
}
