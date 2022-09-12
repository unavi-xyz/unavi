import { useRouter } from "next/router";
import { useState } from "react";

import { trpc } from "../../../src/auth/trpc";
import { getNavbarLayout } from "../../../src/home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../src/home/layouts/ProjectLayout/ProjectLayout";
import Button from "../../../src/ui/base/Button";

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data, isFetched } = trpc.useQuery(["project", { id }], {
    enabled: id !== undefined,
  });

  const { mutateAsync: deleteProject } = trpc.useMutation("delete-project");

  const [loading, setLoading] = useState(false);

  async function handleDelete() {
    setLoading(true);
    await deleteProject({ id });
    router.push("/create");
  }

  if (!isFetched || !data) return null;

  return (
    <ProjectLayout name={data.name} image={data.image}>
      <div className="bg-errorContainer text-onErrorContainer rounded-2xl p-8 space-y-4">
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
