import { MdAdd } from "react-icons/md";

import Card from "../../../src/ui/Card";
import CardGrid from "../../../src/ui/CardGrid";

export default function Loading() {
  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-4 py-8">
        <div className="text-center text-3xl font-black">Create</div>

        <div className="flex items-center justify-between">
          <div className="text-2xl font-bold">⚒️ Projects</div>

          <button
            disabled
            className="cursor-not-allowed rounded-lg px-5 py-1.5 opacity-40 ring-1 ring-neutral-700 transition"
          >
            <MdAdd className="text-lg" />
          </button>
        </div>

        <CardGrid>
          {Array.from({ length: 4 }).map((_, i) => (
            <Card key={i} loading loadingAnimation={false} />
          ))}
        </CardGrid>
      </div>
    </div>
  );
}
