import { notFound } from "next/navigation";

import AuthProvider from "@/src/client/AuthProvider";
import { fetchProject } from "@/src/server/helpers/fetchProject";
import { Studio } from "@/src/studio/components/Studio";
import StudioUI from "@/src/studio/components/StudioUI";

interface Props {
  params?: { id: string };
}

export default async function Page({ params }: Props) {
  if (!params?.id) notFound();

  const project = await fetchProject(params.id);
  if (!project) notFound();

  return (
    <AuthProvider>
      <Studio
        project={project}
        animationPath="/models"
        defaultAvatar="/models/Robot.vrm"
        skybox="/images/Skybox.jpg"
      >
        <StudioUI project={project} />
      </Studio>
    </AuthProvider>
  );
}
