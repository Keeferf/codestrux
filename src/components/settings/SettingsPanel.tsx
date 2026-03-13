import { useState } from "react";
import { Eye, EyeOff, Check, Trash2 } from "lucide-react";
import type { CreativityKey } from "../../types";
import { CREATIVITY_MODES } from "../../constants";
import { SectionHead } from "../ui";
import { HardwarePanel } from "../hardware";
import { saveToken, deleteToken } from "../../lib/Chat";

interface SettingsPanelProps {
  model: string;
  availableModels: string[];
  creativity: CreativityKey;
  tokenSaved: boolean;
  onModelChange: (model: string) => void;
  onCreativityChange: (key: CreativityKey) => void;
  onTokenSaved: () => void;
  onTokenDeleted: () => void;
}

export function SettingsPanel({
  model,
  creativity,
  tokenSaved,
  onCreativityChange,
  onTokenSaved,
  onTokenDeleted,
}: SettingsPanelProps) {
  const modelLoaded = model.length > 0;
  const [tokenInput, setTokenInput] = useState("");
  const [showToken, setShowToken] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const handleSave = async () => {
    if (!tokenInput.trim()) return;
    setSaving(true);
    setSaveError(null);
    try {
      await saveToken(tokenInput.trim());
      setTokenInput("");
      onTokenSaved();
    } catch (e) {
      setSaveError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    try {
      await deleteToken();
      onTokenDeleted();
    } catch {
      // ignore
    }
  };

  return (
    <aside className="w-60 shrink-0 overflow-y-auto px-4 py-3.5 bg-slate-grey-900 border-l border-slate-grey-800 flex flex-col gap-5">
      {/* HF Token */}
      <div>
        <SectionHead label="hf token" />

        {tokenSaved ? (
          <div className="flex items-center justify-between px-3 py-2 rounded-md bg-moss-green-950/40 border border-moss-green-800">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-moss-green-500 shadow-[0_0_5px_rgba(115,155,115,0.5)]" />
              <span className="font-mono text-xs text-moss-green-500">
                token saved
              </span>
            </div>
            <button
              onClick={handleDelete}
              className="text-slate-grey-600 hover:text-brick-red-400 transition-colors"
              title="Remove token"
            >
              <Trash2 size={12} />
            </button>
          </div>
        ) : (
          <div className="flex flex-col gap-1.5">
            <div className="flex items-center gap-1.5 bg-slate-grey-950 border border-slate-grey-800 rounded-md px-2.5 py-1.5 focus-within:border-indigo-smoke-700 transition-colors">
              <input
                type={showToken ? "text" : "password"}
                value={tokenInput}
                onChange={(e) => setTokenInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSave()}
                placeholder="hf_…"
                className="flex-1 bg-transparent outline-none font-mono text-xs text-parchment-200 placeholder:text-slate-grey-600 min-w-0"
              />
              <button
                onClick={() => setShowToken((v) => !v)}
                className="text-slate-grey-600 hover:text-slate-grey-400 transition-colors"
              >
                {showToken ? <EyeOff size={12} /> : <Eye size={12} />}
              </button>
              <button
                onClick={handleSave}
                disabled={!tokenInput.trim() || saving}
                className="text-slate-grey-600 hover:text-moss-green-500 transition-colors disabled:opacity-30"
                title="Save token"
              >
                <Check size={12} />
              </button>
            </div>
            {saveError && (
              <p className="font-mono text-[11px] text-brick-red-400">
                {saveError}
              </p>
            )}
            <p className="font-body text-[11px] text-slate-grey-600 italic leading-relaxed">
              Stored securely by the OS. Get yours at{" "}
              <a
                href="https://huggingface.co/settings/tokens"
                target="_blank"
                rel="noreferrer"
                className="text-indigo-smoke-500 hover:text-indigo-smoke-400 underline"
              >
                hf.co/settings/tokens
              </a>
              .
            </p>
          </div>
        )}
      </div>

      {/* Model status */}
      <div>
        <SectionHead label="model" />
        <div className="rounded-md bg-slate-grey-950 border border-slate-grey-800 px-3 py-2.5">
          <div className="flex items-center gap-2 mb-1.5">
            <div
              className={`w-1.5 h-1.5 rounded-full flex-shrink-0 transition-all duration-300 ${
                modelLoaded
                  ? "bg-moss-green-500 shadow-[0_0_5px_rgba(115,155,115,0.5)]"
                  : "bg-slate-grey-700"
              }`}
            />
            <span
              className={`font-display text-[11px] uppercase tracking-wide ${
                modelLoaded ? "text-moss-green-600" : "text-slate-grey-600"
              }`}
            >
              {modelLoaded ? "loaded" : "no model"}
            </span>
          </div>
          {modelLoaded ? (
            <p className="font-mono text-xs text-parchment-200 break-all leading-relaxed">
              {model}
            </p>
          ) : (
            <p className="font-body text-xs text-slate-grey-600 italic">
              Search for a model in the header bar to get started.
            </p>
          )}
        </div>
      </div>

      {/* Creativity */}
      <div>
        <SectionHead label="creativity" />
        <div className="flex flex-col gap-1.5">
          {(
            Object.entries(CREATIVITY_MODES) as [
              CreativityKey,
              (typeof CREATIVITY_MODES)[CreativityKey],
            ][]
          ).map(([key, { label, temp, desc }]) => {
            const isActive = creativity === key;
            return (
              <button
                key={key}
                onClick={() => onCreativityChange(key)}
                className={`flex items-center justify-between px-3 py-2.25 rounded-[7px] cursor-pointer transition-all duration-150 text-left border ${
                  isActive
                    ? "bg-indigo-smoke-900/20 border-indigo-smoke-700"
                    : "bg-slate-grey-950 border-slate-grey-800 hover:border-slate-grey-700"
                }`}
              >
                <div>
                  <div
                    className={`font-body text-sm ${isActive ? "font-semibold text-indigo-smoke-400" : "font-normal text-parchment-300"}`}
                  >
                    {label}
                  </div>
                  <div
                    className={`font-body text-xs mt-0.5 ${isActive ? "text-indigo-smoke-500" : "text-slate-grey-500"}`}
                  >
                    {desc}
                  </div>
                </div>
                <div
                  className={`font-mono text-xs shrink-0 ml-2 ${isActive ? "text-indigo-smoke-400" : "text-slate-grey-500"}`}
                >
                  {temp}
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Hardware */}
      <div>
        <SectionHead label="hardware" />
        <HardwarePanel />
      </div>
    </aside>
  );
}
