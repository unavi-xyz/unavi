import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import ButtonTabs, { TabContent } from "../../../../src/ui/ButtonTabs";
import About from "./About";
import Settings from "./Settings";

interface Props {
  owner: string;
  params: { id: string };
}

export default async function Tabs({ owner, params }: Props) {
  const session = await getServerSession();

  const isOwner = session?.address === owner;

  return (
    <>
      {isOwner ? (
        <ButtonTabs titles={["About", "Settings"]}>
          <TabContent value="About">
            {/* @ts-expect-error Server Component */}
            <About params={params} />
          </TabContent>
          <TabContent value="Settings">
            {/* @ts-expect-error Server Component */}
            <Settings params={params} />
          </TabContent>
        </ButtonTabs>
      ) : (
        // @ts-expect-error Server Component
        <About params={params} />
      )}
    </>
  );
}
