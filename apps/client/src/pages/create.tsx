import Link from "next/link";
import { useEffect, useState } from "react";
import { MdAdd } from "react-icons/md";
import { useAccount } from "wagmi";

import { useSession } from "../client/auth/useSession";
import { trpc } from "../client/trpc";
import CreateProjectPage from "../home/CreateProjectPage";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../home/MetaTags";
import Button from "../ui/Button";
import Card from "../ui/Card";
import Dialog from "../ui/Dialog";

export default function Create() {
  const [openCreateProject, setOpenCreateProject] = useState(false);

  const { status: authState } = useSession();
  const { status: accountStatus } = useAccount();
  const { invalidateQueries } = trpc.useContext();

  const { data, status, refetch } = trpc.useQuery(["auth.projects"], {
    enabled: authState === "authenticated",
  });

  useEffect(() => {
    if (authState !== "authenticated" || accountStatus !== "connected") return;
    invalidateQueries(["auth.projects"]);
    refetch();
  }, [invalidateQueries, refetch, authState, accountStatus]);

  const authenticated = authState === "authenticated";

  function handleCreateProject() {
    if (!authenticated) return;
    setOpenCreateProject(true);
  }

  return (
    <>
      <MetaTags title="Create" />

      <Dialog
        open={openCreateProject}
        onClose={() => setOpenCreateProject(false)}
      >
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
              status === "success" && data.length > 0 ? (
                data.map(({ id, name, image }) => (
                  <Link key={id} href={`/project/${id}`} passHref>
                    <div>
                      <Card
                        text={name}
                        image={image}
                        sizes="333px"
                        animateEnter
                      />
                    </div>
                  </Link>
                ))
              ) : (
                <div>
                  No projects found.{" "}
                  <button
                    onClick={handleCreateProject}
                    className="cursor-pointer font-bold text-primary decoration-2 hover:underline"
                  >
                    Click here
                  </button>{" "}
                  to create one.
                </div>
              )
            ) : (
              <div>You need to be signed in to create a project.</div>
            )}
          </div>
        </div>
      </div>
    </>
  );
}

Create.getLayout = getNavbarLayout;
