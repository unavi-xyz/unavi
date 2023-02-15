import Link from "next/link";
import { useEffect, useState } from "react";
import { MdAdd } from "react-icons/md";

import { useSession } from "../client/auth/useSession";
import { trpc } from "../client/trpc";
import { env } from "../env/client.mjs";
import CreateProjectPage from "../home/CreateProjectPage";
import MetaTags from "../home/MetaTags";
import { getNavbarLayout } from "../home/NavbarLayout/NavbarLayout";
import Card from "../ui/Card";
import Dialog from "../ui/Dialog";

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publication/${id}/image.jpg`;
}

export default function Spaces() {
  const [openCreateProject, setOpenCreateProject] = useState(false);

  const { status: authState } = useSession();
  const utils = trpc.useContext();

  const authenticated = authState === "authenticated";

  const {
    data: projects,
    status,
    refetch,
  } = trpc.project.getAll.useQuery(undefined, { enabled: authenticated });

  useEffect(() => {
    if (authState !== "authenticated") return;
    utils.project.getAll.invalidate().then(() => refetch());
  }, [refetch, authState, utils]);

  function handleCreateProject() {
    if (!authenticated) return;
    setOpenCreateProject(true);
  }

  const unpublishedProjects = projects?.filter((p) => !p.Publication?.spaceId) ?? [];
  const publishedProjects = projects?.filter((p) => p.Publication?.spaceId) ?? [];
  const publishedImages = publishedProjects.map((p) =>
    p.publicationId ? cdnImageURL(p.publicationId) : p.image
  );

  return (
    <>
      <MetaTags title="Create" />

      <Dialog open={openCreateProject} onOpenChange={setOpenCreateProject} title="Create Project">
        <CreateProjectPage />
      </Dialog>

      <div className="mx-4 flex justify-center py-8">
        <div className="max-w-content space-y-8">
          <div className="flex justify-center text-3xl font-black">Create</div>

          <div className="flex items-center justify-between">
            <div className="text-2xl font-bold">⚒️ Projects</div>
            <div>
              <button
                disabled={!authenticated}
                onClick={handleCreateProject}
                className={`rounded-lg px-5 py-1.5 ring-1 ring-neutral-700 transition ${
                  authenticated
                    ? "hover:bg-neutral-200 active:opacity-80"
                    : "cursor-not-allowed opacity-40"
                }`}
              >
                <MdAdd className="text-lg" />
              </button>
            </div>
          </div>

          <div className="grid grid-cols-4 gap-3">
            {authenticated ? (
              status === "success" && unpublishedProjects.length > 0 ? (
                unpublishedProjects.map(({ id, name, image }) => (
                  <Link key={id} href={`/project/${id}`}>
                    <Card text={name} image={image} sizes="333px" animateEnter />
                  </Link>
                ))
              ) : (
                <div className="col-span-4">
                  No projects found.{" "}
                  <button
                    onClick={handleCreateProject}
                    className="cursor-pointer font-bold text-neutral-900 decoration-2 hover:underline"
                  >
                    Click here
                  </button>{" "}
                  to create one.
                </div>
              )
            ) : (
              <div className="col-span-4 text-neutral-500">
                You need to be signed in to create a project.
              </div>
            )}
          </div>

          {authState === "authenticated" && status === "success" && publishedProjects.length > 0 ? (
            <>
              <div className="text-2xl font-bold">🌍 Published</div>

              <div className="grid grid-cols-4 gap-3">
                {publishedProjects.map(({ id, name }, i) => (
                  <Link key={id} href={`/project/${id}`}>
                    <Card text={name} image={publishedImages[i]} sizes="333px" animateEnter />
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
