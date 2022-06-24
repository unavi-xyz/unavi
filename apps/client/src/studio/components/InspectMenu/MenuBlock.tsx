import { useStudioStore } from "../../../studio/store";
import CollapseMenu from "../../../ui/base/CollapseMenu";

interface Props {
  menuId: string;
  title?: string;
  children: React.ReactNode;
}

export default function MenuBlock({ menuId, title, children }: Props) {
  const closedInspectMenus = useStudioStore(
    (state) => state.closedInspectMenus
  );

  return (
    <CollapseMenu
      open={!closedInspectMenus.includes(menuId)}
      toggle={() => useStudioStore.getState().toggleClosedInspectMenu(menuId)}
      title={title ?? menuId}
    >
      <div className="space-y-1">{children}</div>
    </CollapseMenu>
  );
}
