import { useState } from "react";
import { MdAdd } from "react-icons/md";

import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import CreateScenePage from "../src/ui/CreateScenePage";
import MetaTags from "../src/ui/MetaTags";
import Button from "../src/ui/base/Button";
import Card from "../src/ui/base/Card";
import Dialog from "../src/ui/base/Dialog";

export default function Create() {
  const [openCreateSpace, setOpenCreateSpace] = useState(false);

  return (
    <>
      <MetaTags title="Create" />

      <Dialog open={openCreateSpace} onClose={() => setOpenCreateSpace(false)}>
        <CreateScenePage />
      </Dialog>

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex justify-center font-black text-3xl">Create</div>

          <div className="flex items-center justify-between">
            <div className="text-2xl font-bold">Scenes</div>
            <div>
              <Button variant="outlined" squared="small" onClick={() => setOpenCreateSpace(true)}>
                <MdAdd className="text-lg" />
              </Button>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-2">
            <Card />
            <Card />
          </div>
        </div>
      </div>
    </>
  );
}

Create.getLayout = getNavbarLayout;
