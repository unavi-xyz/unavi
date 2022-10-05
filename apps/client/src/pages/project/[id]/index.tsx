import { useRouter } from "next/router";

import { trpc } from "../../../auth/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../home/layouts/ProjectLayout/ProjectLayout";

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.useQuery(["auth.project", { id }], {
    enabled: id !== undefined,
  });

  const { data: imageURL } = trpc.useQuery(["auth.project-image", { id }], {
    enabled: id !== undefined,
  });

  return (
    <ProjectLayout name={project?.name} image={imageURL}>
      <div>{project?.description}</div>
    </ProjectLayout>
  );
}

Project.getLayout = getNavbarLayout;
