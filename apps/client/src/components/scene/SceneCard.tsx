import { useLocalScene } from "./localScenes/useLocalScene";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const localScene = useLocalScene(id);

  return (
    <div className="w-full h-40">
      <div
        className="relative w-full h-full rounded-lg hover:cursor-pointer
                   flex flex-col border"
      >
        <div className="h-full">
          {localScene?.image && (
            <img
              src={localScene.image}
              alt="scene image"
              className="w-full h-full object-cover rounded-lg"
            />
          )}
        </div>

        {localScene?.name && (
          <div className="absolute border-t bottom-0 px-3 py-2 rounded-b-lg w-full bg-neutral-50">
            {localScene.name}
          </div>
        )}
      </div>
    </div>
  );
}
