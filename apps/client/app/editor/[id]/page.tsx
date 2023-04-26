import { Metadata } from "next";
import { notFound } from "next/navigation";

import AuthProvider from "@/src/client/AuthProvider";
import { Editor } from "@/src/editor/components/Editor";
import EditorUI from "@/src/editor/components/EditorUI";
import { fetchProject } from "@/src/server/helpers/fetchProject";

import RainbowkitWrapper from "../../(navbar)/RainbowkitWrapper";

type Params = {
  id: string;
};

export async function generateMetadata({ params: { id } }: { params: Params }): Promise<Metadata> {
  const project = await fetchProject(id);

  if (!project) return {};

  return {
    title: project.title,
    description: project.description,
  };
}

export default async function Page({ params: { id } }: { params: Params }) {
  const project = await fetchProject(id);

  if (!project) notFound();

  return (
    <AuthProvider>
      <RainbowkitWrapper>
        <Editor
          project={project}
          animationPath="/models"
          defaultAvatar="/models/Robot.vrm"
          skybox="/images/Skybox.jpg"
        >
          <EditorUI project={project} />
        </Editor>
      </RainbowkitWrapper>
    </AuthProvider>
  );
}
