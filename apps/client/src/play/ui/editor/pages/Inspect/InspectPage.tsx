import { useTreeItem } from "../../hooks/useTreeItem";
import PanelPage from "../PanelPage";

interface Props {
  id: bigint;
}

export default function InspectPage({ id }: Props) {
  const item = useTreeItem(id);

  if (!item) {
    return null;
  }

  return (
    <PanelPage title={item.name || `(${id})`}>
      <div></div>
    </PanelPage>
  );
}
