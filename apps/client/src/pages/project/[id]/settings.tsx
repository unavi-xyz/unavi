import { useRouter } from "next/router";
import { useState } from "react";

import { trpc } from "../../../client/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../home/layouts/ProjectLayout/ProjectLayout";
import Button from "../../../ui/Button";

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.useQuery(["auth.project", { id }], {
    enabled: id !== undefined,
  });

  const { data: imageURL } = trpc.useQuery(["auth.project-image", { id }], {
    enabled: id !== undefined,
  });

  const { mutateAsync: deleteProject } = trpc.useMutation(
    "auth.delete-project"
  );

  const [loading, setLoading] = useState(false);

  async function handleDelete() {
    setLoading(true);
    await deleteProject({ id });
    router.push("/create");
  }

  return (
    <ProjectLayout name={project?.name} image={imageURL}>
      <div className="space-y-4 rounded-2xl bg-errorContainer p-8 text-onErrorContainer">
        <div className="text-2xl font-bold">Danger Zone</div>

        <div className="text-lg">
          Deleting a project is permanent and cannot be undone.
        </div>

        <Button
          variant="filled"
          color="error"
          rounded="large"
          loading={loading}
          onClick={handleDelete}
        >
          Delete Project
        </Button>
      </div>
    </ProjectLayout>
  );
}

Project.getLayout = getNavbarLayout;
