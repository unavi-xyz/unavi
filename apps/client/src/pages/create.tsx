import Link from "next/link";
import { useEffect, useState } from "react";
import { MdAdd } from "react-icons/md";

import { useSession } from "../client/auth/useSession";
import { trpc } from "../client/trpc";
import { env } from "../env/client.mjs";
import CreateProjectPage from "../home/CreateProjectPage";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../home/MetaTags";
import Button from "../ui/Button";
import Card from "../ui/Card";
import Dialog from "../ui/Dialog";

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/image.jpg`;
}

export default function Spaces() {
  const [openCreateProject, setOpenCreateProject] = useState(false);

  const { status: authState } = useSession();
  const utils = trpc.useContext();

  const {
    data: projects,
    status,
    refetch,
  } = trpc.project.getAll.useQuery(undefined, {
    enabled: authState === "authenticated",
  });

  useEffect(() => {
    if (authState !== "authenticated") return;
    utils.project.getAll.invalidate().then(() => refetch());
  }, [refetch, authState, utils]);

  const authenticated = authState === "authenticated";

  function handleCreateProject() {
    if (!authenticated) return;
    setOpenCreateProject(true);
  }

  const unpublishedProjects = projects?.filter((p) => !p.publicationId) ?? [];
  const publishedProjects = projects?.filter((p) => p.publicationId) ?? [];
  const publishedImages = publishedProjects.map((p) =>
    p.publicationId ? cdnImageURL(p.publicationId) : p.image
  );

  return (
    <>
      <MetaTags title="Create" />

      <Dialog open={openCreateProject} onClose={() => setOpenCreateProject(false)}>
        <CreateProjectPage />
      </Dialog>

      <div className="mx-4 flex justify-center py-8">
        <div className="max-w-content space-y-8">
          <div className="flex justify-center text-3xl font-black">Create</div>

          <div className="flex items-center justify-between">
            <div className="text-2xl font-bold">⚒️ Projects</div>
            <div>
              <Button
                variant="outlined"
                rounded="small"
                disabled={!authenticated}
                onClick={handleCreateProject}
              >
                <MdAdd className="text-lg" />
              </Button>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-3">
            {authState === "authenticated" ? (
              status === "success" && unpublishedProjects.length > 0 ? (
                unpublishedProjects.map(({ id, name, image }) => (
                  <Link key={id} href={`/project/${id}`}>
                    <div>
                      <Card text={name} image={image} sizes="333px" animateEnter />
                    </div>
                  </Link>
                ))
              ) : (
                <div>
                  No projects found.{" "}
                  <button
                    onClick={handleCreateProject}
                    className="cursor-pointer font-bold text-sky-400 decoration-2 hover:underline"
                  >
                    Click here
                  </button>{" "}
                  to create one.
                </div>
              )
            ) : (
              <div className="text-neutral-500">You need to be signed in to create a project.</div>
            )}
          </div>

          {authState === "authenticated" && status === "success" && publishedProjects.length > 0 ? (
            <>
              <div className="text-2xl font-bold">🏙️ Published</div>

              <div className="grid grid-cols-3 gap-3">
                {publishedProjects.map(({ id, name }, i) => (
                  <Link key={id} href={`/project/${id}`}>
                    <div>
                      <Card text={name} image={publishedImages[i]} sizes="333px" animateEnter />
                    </div>
                  </Link>
                ))}
              </div>
            </>
          ) : null}
        </div>
      </div>
    </>
  );
}

Spaces.getLayout = getNavbarLayout;
