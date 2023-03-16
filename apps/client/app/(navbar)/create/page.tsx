import { Metadata } from "next";

import { metadata as baseMetadata } from "../../layout";
import CreateProjectButton from "./CreateProjectButton";
import Projects from "./Projects";
import Published from "./Published";

const TITLE = "Create";

export const metadata: Metadata = {
  title: TITLE,
  openGraph: {
    ...baseMetadata.openGraph,
    title: TITLE,
  },
  twitter: {
    ...baseMetadata.twitter,
    title: TITLE,
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

        {/* @ts-expect-error Server Component */}
        <Projects />

        {/* @ts-expect-error Server Component */}
        <Published />
      </div>
    </div>
  );
}
