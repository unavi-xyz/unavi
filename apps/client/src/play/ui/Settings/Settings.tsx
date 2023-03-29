import AccountSettings from "./AccountSettings";
import AvatarSettings from "./AvatarSettings";
import NameSettings from "./NameSettings";
import { SettingsPage } from "./SettingsDialog";

interface Props {
  onClose: () => void;
  setPage: (page: SettingsPage) => void;
}

export default function Settings({ onClose, setPage }: Props) {
  return (
    <div className="space-y-4">
      <NameSettings />
      <AvatarSettings setPage={setPage} />
      <AccountSettings onClose={onClose} />
    </div>
  );
}
