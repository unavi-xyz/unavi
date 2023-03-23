import { Metadata } from "next";
import { notFound } from "next/navigation";

import { fetchProject } from "../../../src/server/helpers/fetchProject";
import RainbowkitWrapper from "../../(navbar)/RainbowkitWrapper";
import SessionProvider from "../../(navbar)/SessionProvider";
import Editor from "./Editor";

type Params = {
  id: string;
};

export async function generateMetadata({ params: { id } }: { params: Params }): Promise<Metadata> {
  const project = await fetchProject(id);

  if (!project) return {};

  return {
    title: project.name,
  };
}

export default async function Page({ params: { id } }: { params: Params }) {
  const project = await fetchProject(id);

  if (!project) notFound();

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <Editor project={project} />
      </RainbowkitWrapper>
    </SessionProvider>
  );
}
