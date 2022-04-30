import { useStudioStore } from "../../helpers/studio/store";

export default function InspectMenu() {
  const selected = useStudioStore((state) => state.selected);

  if (!selected) return <div></div>;

  return (
    <div className="p-4 space-y-4 w-full">
      <div className="flex justify-center text-xl font-bold">
        {selected.name}
      </div>

      <div>
        <div></div>
      </div>
    </div>
  );
}
