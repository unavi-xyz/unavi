import { useRouter } from "next/router";
import { useState } from "react";

import { trpc } from "../../../auth/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../home/layouts/ProjectLayout/ProjectLayout";
import Button from "../../../ui/base/Button";

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data, isFetched } = trpc.useQuery(["auth.project", { id }], {
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

  if (!isFetched || !data) return null;

  return (
    <ProjectLayout name={data.name} image={data.image}>
      <div className="bg-errorContainer text-onErrorContainer space-y-4 rounded-2xl p-8">
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
