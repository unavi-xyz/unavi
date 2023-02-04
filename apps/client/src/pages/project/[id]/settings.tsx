import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useRef, useState } from "react";
import { toast } from "react-hot-toast";

import { useSession } from "../../../client/auth/useSession";
import { trpc } from "../../../client/trpc";
import { env } from "../../../env/client.mjs";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../home/layouts/ProjectLayout/ProjectLayout";
import Button from "../../../ui/Button";
import { hexDisplayToNumber, numberToHexDisplay } from "../../../utils/numberToHexDisplay";

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publication/${id}/image.jpg`;
}

export default function Project() {
  const utils = trpc.useContext();
  const router = useRouter();
  const id = router.query.id as string;

  const spaceIdRef = useRef<HTMLInputElement>(null);

  const [loadingDelete, setLoadingDelete] = useState(false);
  const [loadingConnect, setLoadingConnect] = useState(false);

  const { mutateAsync: publish } = trpc.project.publish.useMutation();
  const { mutateAsync: update } = trpc.project.update.useMutation();
  const { mutateAsync: deleteProject } = trpc.project.delete.useMutation();
  const { data: project } = trpc.project.get.useQuery({ id }, { enabled: id !== undefined });
  const { data: imageURL } = trpc.project.image.useQuery(
    { id },
    { enabled: id !== undefined && !project?.publicationId }
  );

  const { data: session } = useSession();

  useEffect(() => {
    if (!spaceIdRef.current || !project) return;
    const spaceId = project.Publication?.spaceId;
    const hexId = spaceId ? numberToHexDisplay(spaceId) : null;
    spaceIdRef.current.value = hexId ?? "";
  }, [spaceIdRef, project]);

  async function handleConnect() {
    if (!session?.address || loadingConnect) return;

    const hexId = spaceIdRef.current?.value ?? "";
    const spaceId = hexDisplayToNumber(hexId);

    if (!hexId) {
      // Disconnect project
      await toast.promise(update({ id, publicationId: null }), {
        loading: "Disconnecting project...",
        success: "Project disconnected",
        error: "Failed to disconnect project",
      });

      await utils.project.get.invalidate({ id });
      return;
    }

    if (Number.isNaN(spaceId)) {
      toast.error("Invalid space ID");
      return;
    }

    let error = "Failed to connect project";

    async function connect() {
      if (!session?.address) return;

      // Fetch space
      const space = await utils.space.byId.fetch({ id: spaceId });

      if (!space) {
        error = "Space not found";
        throw new Error(error);
      }

      if (space.owner !== session.address) {
        error = "You do not own this space";
        throw new Error(error);
      }

      // Fetch publication
      const publication = await utils.publication.bySpaceId.fetch({ spaceId });
      let publicationId = publication?.id;

      if (!publicationId) {
        // Create new publication
        publicationId = await publish({ id });
      }

      // Link project to publication
      await update({ id, publicationId });
    }

    setLoadingConnect(true);

    await toast.promise(
      connect().finally(() => setLoadingConnect(false)),
      {
        loading: "Connecting project...",
        success: "Project connected",
        error: () => error,
      }
    );

    await utils.project.get.invalidate({ id });
  }

  function handleDelete() {
    if (loadingDelete) return;
    setLoadingDelete(true);

    const promises: Promise<unknown>[] = [];
    promises.push(deleteProject({ id }));
    promises.push(utils.project.getAll.invalidate());

    toast
      .promise(Promise.all(promises), {
        loading: "Deleting project...",
        success: "Project deleted",
        error: "Failed to delete project",
      })
      .then(() => router.push("/create"))
      .catch((err) => console.error(err))
      .finally(() => setLoadingDelete(false));
  }

  // If published, use the published image
  const image = project?.publicationId ? cdnImageURL(project.publicationId) : imageURL;

  return (
    <ProjectLayout name={project?.name} image={image}>
      <div className="space-y-8">
        <div className="space-y-2 rounded-2xl p-8">
          <div className="text-2xl font-bold">Connect Space</div>
          <div className="pb-1 text-lg text-neutral-500">
            Connecting your project to a published space will allow you to push updates to it. This
            is done automatically when you first publish your project, or you can set the space ID
            manually here.
          </div>

          <div className="flex items-center space-x-2">
            <input
              ref={spaceIdRef}
              type="text"
              placeholder="0x01"
              onChange={(e) => {
                // Limit to valid hex characters + 0x
                const value = e.target.value.replace(/[^0-9a-fA-FxX]/g, "").slice(0, 8);
                e.target.value = value;
              }}
              className="h-9 w-24 rounded-lg py-1 pr-1 text-center ring-1 ring-inset ring-neutral-500"
            />

            <Button disabled={loadingConnect} onClick={handleConnect} className="h-9 rounded-lg">
              Connect
            </Button>

            {project?.Publication?.spaceId && (
              <Link
                href={`/space/${numberToHexDisplay(project?.Publication?.spaceId)}`}
                className="flex h-9 items-center justify-center rounded-lg px-4 font-bold hover:bg-neutral-200 active:opacity-80"
              >
                View
              </Link>
            )}
          </div>
        </div>

        <div className="space-y-2 rounded-2xl bg-red-100 px-8 py-6 text-red-900 ring-2 ring-inset ring-red-900/20">
          <div className="text-2xl font-bold">Danger Zone</div>
          <div className="pb-1 text-lg">Deleting a project is permanent and cannot be undone.</div>

          <Button disabled={loadingDelete} onClick={handleDelete} className="rounded-lg bg-red-700">
            Delete Project
          </Button>
        </div>
      </div>
    </ProjectLayout>
  );
}

Project.getLayout = getNavbarLayout;
