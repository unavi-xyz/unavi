import { getServerSession } from "@/src/server/helpers/getServerSession";
import ButtonTabs, { TabContent } from "@/src/ui/ButtonTabs";

import About from "./About";
import Settings from "./Settings";

interface Props {
  id: string;
  owner: string;
  description: string;
}

export default async function Tabs({ id, owner, description }: Props) {
  const session = await getServerSession();

  const isOwner = session?.address === owner;

  return (
    <>
      {isOwner ? (
        <ButtonTabs titles={["About", "Settings"]}>
          <TabContent value="About">
            {/* @ts-expect-error Server Component */}
            <About description={description} />
          </TabContent>
          <TabContent value="Settings">
            {/* @ts-expect-error Server Component */}
            <Settings id={id} />
          </TabContent>
        </ButtonTabs>
      ) : (
        // @ts-expect-error Server Component
        <About description={description} />
      )}
    </>
  );
}
