import { Metadata } from "next";
import { Suspense } from "react";

import { metadata as baseMetadata } from "../../layout";
import CreateProjectButton from "./CreateProjectButton";
import Projects from "./Projects";
import Published from "./Published";

const title = "Create";

export const metadata: Metadata = {
  title,
  openGraph: {
    ...baseMetadata.openGraph,
    title,
  },
  twitter: {
    ...baseMetadata.twitter,
    title,
  },
};

export default function Create() {
  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-4 py-8">
        <div className="text-center text-3xl font-black">Create</div>

        <div className="flex items-center justify-between">
          <div className="text-2xl font-bold">⚒️ Projects</div>

          {/* @ts-expect-error Server Component */}
          <CreateProjectButton />
        </div>

        <Suspense fallback={null}>
          {/* @ts-expect-error Server Component */}
          <Projects />
        </Suspense>

        <Suspense fallback={null}>
          {/* @ts-expect-error Server Component */}
          <Published />
        </Suspense>
      </div>
    </div>
  );
}
