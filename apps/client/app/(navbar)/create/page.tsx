import { Suspense } from "react";
import { MdAdd } from "react-icons/md";

import Card from "../../../src/ui/Card";
import CreateProjectButton from "./CreateProjectButton";
import Projects from "./Projects";
import Published from "./Published";

export const metadata = {
  title: "Create",
};

export default function Create() {
  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-4 py-8">
        <div className="text-center text-3xl font-black">Create</div>

        <div className="flex items-center justify-between">
          <div className="text-2xl font-bold">⚒️ Projects</div>

          <Suspense
            fallback={
              <button
                disabled
                className="cursor-not-allowed rounded-lg px-5 py-1.5 opacity-40 ring-1 ring-neutral-700 transition"
              >
                <MdAdd className="text-lg" />
              </button>
            }
          >
            {/* @ts-expect-error Server Component */}
            <CreateProjectButton />
          </Suspense>
        </div>

        <div className="grid w-full grid-cols-2 gap-3 md:grid-cols-4">
          <Suspense fallback={new Array(4).fill(<Card loading />)}>
            {/* @ts-expect-error Server Component */}
            <Projects />
          </Suspense>
        </div>

        <Suspense fallback={null}>
          {/* @ts-expect-error Server Component */}
          <Published />
        </Suspense>
      </div>
    </div>
  );
}
