import useLocalWorld from "./localWorlds/useLocalWorld";

interface Props {
  id: string;
}

export default function LocalWorldCard({ id }: Props) {
  const world = useLocalWorld(id);

  return (
    <div className="w-64 h-48 p-4 bg-neutral-200 rounded hover:cursor-pointer flex flex-col justify-end">
      <div>{world?.name}</div>
    </div>
  );
}
