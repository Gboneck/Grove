import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Modal from "../Modal";

interface PluginInfo {
  name: string;
  version: string;
  description: string;
  enabled: boolean;
  actions_count: number;
  blocks_count: number;
  data_sources_count: number;
}

interface PluginPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

async function listPlugins(): Promise<PluginInfo[]> {
  return invoke<PluginInfo[]>("list_plugins");
}

async function setPluginEnabled(name: string, enabled: boolean): Promise<void> {
  return invoke<void>("set_plugin_enabled", { name, enabled });
}

export default function PluginPanel({ isOpen, onClose }: PluginPanelProps) {
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = () => {
    setLoading(true);
    listPlugins()
      .then(setPlugins)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    if (isOpen) refresh();
  }, [isOpen]);

  const toggle = async (name: string, currentEnabled: boolean) => {
    await setPluginEnabled(name, !currentEnabled);
    refresh();
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Plugins" maxWidth="max-w-lg">
      <div className="px-6 py-4 space-y-3">
        {loading && plugins.length === 0 ? (
          <p className="text-sm text-grove-text-secondary">Loading...</p>
        ) : plugins.length === 0 ? (
          <div className="space-y-3">
            <p className="text-sm text-grove-text-secondary">No plugins installed.</p>
            <div className="bg-grove-surface border border-grove-border rounded-lg p-4">
              <p className="text-xs text-grove-text-secondary">
                Add plugins by creating <code className="font-mono text-grove-text-primary">.toml</code> files in{" "}
                <code className="font-mono text-grove-text-primary">~/.grove/plugins/</code>
              </p>
              <p className="text-xs text-grove-text-secondary mt-2">
                See <code className="font-mono text-grove-text-primary">_example.toml.disabled</code> for the format.
              </p>
            </div>
          </div>
        ) : (
          plugins.map((plugin) => (
            <div
              key={plugin.name}
              className={`bg-grove-surface border border-grove-border rounded-lg p-4 transition-opacity ${
                !plugin.enabled ? "opacity-50" : ""
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-grove-text-primary">{plugin.name}</span>
                    <span className="text-[10px] font-mono text-grove-text-secondary">v{plugin.version}</span>
                  </div>
                  {plugin.description && (
                    <p className="text-xs text-grove-text-secondary">{plugin.description}</p>
                  )}
                  <div className="flex gap-3 text-[10px] text-grove-text-secondary">
                    {plugin.actions_count > 0 && <span>{plugin.actions_count} actions</span>}
                    {plugin.blocks_count > 0 && <span>{plugin.blocks_count} blocks</span>}
                    {plugin.data_sources_count > 0 && <span>{plugin.data_sources_count} sources</span>}
                  </div>
                </div>
                <button
                  onClick={() => toggle(plugin.name, plugin.enabled)}
                  className={`shrink-0 w-10 h-5 rounded-full transition-colors relative ${
                    plugin.enabled ? "bg-grove-accent" : "bg-grove-border"
                  }`}
                  aria-label={`${plugin.enabled ? "Disable" : "Enable"} ${plugin.name}`}
                >
                  <span
                    className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                      plugin.enabled ? "left-5" : "left-0.5"
                    }`}
                  />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="px-6 py-3 border-t border-grove-border">
        <p className="text-[10px] text-grove-text-secondary">
          Plugins add actions, block types, and data sources to Grove. Changes take effect on next reasoning cycle.
        </p>
      </div>
    </Modal>
  );
}
