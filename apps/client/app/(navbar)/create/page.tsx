import { Suspense } from "react";

import { getServerSession } from "../../../src/server/helpers/getServerSession";
import Card from "../../../src/ui/Card";
import CreateProjectButton from "./CreateProjectButton";
import Projects from "./Projects";
import Published from "./Published";

export const metadata = {
  title: "Create",
};

export default async function Create() {
  const session = await getServerSession();

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-4 py-8">
        <div className="text-center text-3xl font-black">Create</div>

        <div className="flex items-center justify-between">
          <div className="text-2xl font-bold">⚒️ Projects</div>
          {/* @ts-expect-error Server Component */}
          <CreateProjectButton />
        </div>

        <div className="grid w-full grid-cols-2 gap-3 md:grid-cols-4">
          {session ? (
            <Suspense fallback={new Array(4).fill(<Card loading />)}>
              {/* @ts-expect-error Server Component */}
              <Projects />
            </Suspense>
          ) : (
            <div className="text-neutral-500">You must be signed in to create a project.</div>
          )}
        </div>

        {session ? (
          <Suspense fallback={null}>
            {/* @ts-expect-error Server Component */}
            <Published />
          </Suspense>
        ) : null}
      </div>
    </div>
  );
}
