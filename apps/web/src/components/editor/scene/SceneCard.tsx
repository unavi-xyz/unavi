import { useLocalScene } from "./localScenes/useLocalScene";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const localScene = useLocalScene(id);

  return (
    <div className="w-full h-40">
      <div
        className="relative w-full h-full rounded-xl hover:cursor-pointer
                   flex flex-col"
      >
        <div className="h-full">
          {localScene?.image && (
            <img
              src={localScene.image}
              alt="scene image"
              className="w-full h-full object-cover rounded-xl opacity-100"
            />
          )}
        </div>

        {localScene?.name && (
          <div className="absolute -mb-px bottom-0 px-3 py-2 text-lg rounded-b-xl w-full bg-neutral-100">
            {localScene.name}
          </div>
        )}
      </div>
    </div>
  );
}
