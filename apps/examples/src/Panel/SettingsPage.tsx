import { Settings } from "../ExampleCanvas";

interface Props {
  settings: Settings;
  setSettings: (settings: Settings) => void;
}

export default function SettingsPage({ settings, setSettings }: Props) {
  return (
    <div className="space-y-1">
      <div className="grid grid-cols-2">
        <div>Test Three Loader?</div>
        <div>
          <input
            type="checkbox"
            checked={settings.testThree}
            onChange={(e) =>
              setSettings({ ...settings, testThree: e.target.checked })
            }
          />
        </div>
      </div>

      <div className="grid grid-cols-2">
        <div>Test Exporter?</div>
        <div>
          <input
            type="checkbox"
            checked={settings.testExport}
            onChange={(e) =>
              setSettings({ ...settings, testExport: e.target.checked })
            }
          />
        </div>
      </div>
    </div>
  );
}
