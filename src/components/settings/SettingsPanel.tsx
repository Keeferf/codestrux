import type { CreativityKey } from "../../types";
import { CREATIVITY_MODES } from "../../constants";
import { SectionHead } from "../ui";

interface SettingsPanelProps {
  model: string;
  availableModels: string[];
  creativity: CreativityKey;
  onModelChange: (model: string) => void;
  onCreativityChange: (key: CreativityKey) => void;
}

export function SettingsPanel({
  model,
  availableModels,
  creativity,
  onModelChange,
  onCreativityChange,
}: SettingsPanelProps) {
  return (
    <aside className="w-60 shrink-0 overflow-y-auto px-4 py-3.5 bg-slate-grey-900 border-l border-slate-grey-800">
      <SectionHead label="model" />
      <div className="mb-5">
        <p className="font-display text-[11px] uppercase tracking-wide mb-1.5 text-slate-grey-500">
          Loaded Model
        </p>
        {availableModels.length === 0 ? (
          <div className="w-full rounded px-2 py-1.5 font-body text-sm bg-slate-grey-950 border border-slate-grey-800 text-slate-grey-500">
            no models found
          </div>
        ) : (
          <select
            value={model}
            onChange={(e) => onModelChange(e.target.value)}
            className="w-full rounded px-2 py-1.5 font-mono text-xs outline-none cursor-pointer bg-slate-grey-950 border border-slate-grey-800 text-parchment-200 hover:border-slate-grey-700 focus:border-indigo-smoke-600 transition-colors"
          >
            {availableModels.map((m) => (
              <option
                key={m}
                value={m}
                className="bg-slate-grey-950 text-parchment-200"
              >
                {m}
              </option>
            ))}
          </select>
        )}
      </div>

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
                  className={`font-body text-sm ${
                    isActive
                      ? "font-semibold text-indigo-smoke-400"
                      : "font-normal text-parchment-300"
                  }`}
                >
                  {label}
                </div>
                <div
                  className={`font-body text-xs mt-0.5 ${
                    isActive ? "text-indigo-smoke-500" : "text-slate-grey-500"
                  }`}
                >
                  {desc}
                </div>
              </div>
              <div
                className={`font-mono text-xs shrink-0 ml-2 ${
                  isActive ? "text-indigo-smoke-400" : "text-slate-grey-500"
                }`}
              >
                {temp}
              </div>
            </button>
          );
        })}
      </div>
    </aside>
  );
}
