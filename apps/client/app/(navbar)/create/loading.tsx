import { MdAdd } from "react-icons/md";

import Card from "../../../src/ui/Card";

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

        <div className="grid w-full grid-cols-2 gap-3 md:grid-cols-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <Card key={i} loading loadingAnimation={false} />
          ))}
        </div>
      </div>
    </div>
  );
}
