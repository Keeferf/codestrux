import {
  Trash2,
  HardDrive,
  X,
  Play,
  Square,
  Cpu,
  Zap,
  Loader2,
} from "lucide-react";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CreativityKey } from "../../types";
import { CREATIVITY_MODES } from "../../constants";
import { SectionHead } from "../ui";
import { HardwarePanel } from "../hardware";
import { deleteDownloadedModel, formatBytes } from "../../lib/Download";
import type { DownloadedModel, DownloadProgress } from "../../lib/Download";

// ── Types ─────────────────────────────────────────────────────────────────────

interface LoadedModelInfo {
  model_id: string;
  filename: string;
  /** "vulkan" | "cpu" */
  backend: string;
}

type LoadingState =
  | { status: "idle" }
  | { status: "loading"; backend: string | null };

// ── Props ─────────────────────────────────────────────────────────────────────

interface SettingsPanelProps {
  creativity: CreativityKey;
  downloadedModels: DownloadedModel[];
  activeDownload: DownloadProgress | null;
  onCreativityChange: (key: CreativityKey) => void;
  onCancelDownload: () => void;
  onModelsChanged: () => void;
}

// ── Backend badge ─────────────────────────────────────────────────────────────

function BackendBadge({ backend }: { backend: string }) {
  const isVulkan = backend === "vulkan";
  return (
    <span
      className={`inline-flex items-center gap-1 font-mono text-[9px] px-1.5 py-0.5 rounded-full border ${
        isVulkan
          ? "bg-emerald-950/40 border-emerald-800/50 text-emerald-400"
          : "bg-slate-grey-800/60 border-slate-grey-700 text-slate-grey-400"
      }`}
      title={isVulkan ? "Running on GPU via Vulkan" : "Running on CPU"}
    >
      {isVulkan ? <Zap size={8} /> : <Cpu size={8} />}
      {backend}
    </span>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export function SettingsPanel({
  creativity,
  downloadedModels,
  activeDownload,
  onCreativityChange,
  onCancelDownload,
  onModelsChanged,
}: SettingsPanelProps) {
  const [loadedModel, setLoadedModel] = useState<LoadedModelInfo | null>(null);
  const [loadingState, setLoadingState] = useState<LoadingState>({
    status: "idle",
  });

  // ── Sync loaded-model state on mount ─────────────────────────────────────
  useEffect(() => {
    invoke<LoadedModelInfo | null>("get_loaded_model")
      .then((info) => setLoadedModel(info ?? null))
      .catch(() => {});
  }, []);

  // ── Listen for backend events from Rust ──────────────────────────────────
  useEffect(() => {
    const unlisten: Array<Promise<() => void>> = [];

    unlisten.push(
      listen<{ backend: string }>("model-backend-trying", (e) => {
        setLoadingState({ status: "loading", backend: e.payload.backend });
      }),
    );

    unlisten.push(
      listen<LoadedModelInfo>("model-loaded", (e) => {
        setLoadedModel(e.payload);
        setLoadingState({ status: "idle" });
      }),
    );

    // model-error, model-backend-failed, and model-debug are handled silently.
    unlisten.push(
      listen("model-error", () => {
        setLoadingState({ status: "idle" });
      }),
    );

    unlisten.push(listen("model-debug", (_e) => {}));

    return () => {
      unlisten.forEach((p) => p.then((fn) => fn()));
    };
  }, []);

  // ── Handlers ──────────────────────────────────────────────────────────────

  const handleDeleteModel = async (modelId: string, filename: string) => {
    try {
      await deleteDownloadedModel(modelId, filename);
      // If the deleted model is currently loaded, clear local state too.
      if (
        loadedModel?.model_id === modelId &&
        loadedModel?.filename === filename
      ) {
        setLoadedModel(null);
      }
      onModelsChanged();
    } catch {
      /* ignore */
    }
  };

  const handleLoadModel = async (modelId: string, filename: string) => {
    setLoadingState({ status: "loading", backend: null });
    try {
      await invoke("load_local_model", { modelId, filename });
      // `model-loaded` event updates state; no need to do it here too.
    } catch {
      setLoadingState({ status: "idle" });
    }
  };

  const handleUnloadModel = async () => {
    try {
      await invoke("unload_local_model");
      setLoadedModel(null);
      setLoadingState({ status: "idle" });
    } catch {
      /* ignore */
    }
  };

  const isLoading = loadingState.status === "loading";
  const isLoaded = (modelId: string, filename: string) =>
    loadedModel?.model_id === modelId && loadedModel?.filename === filename;

  return (
    <aside className="w-60 shrink-0 overflow-y-auto px-4 py-3.5 bg-slate-grey-900 border-l border-slate-grey-800 flex flex-col gap-5">
      {/* ── Active download progress ────────────────────────────────────── */}
      {activeDownload && (
        <div>
          <SectionHead label="downloading" />
          <div className="rounded-md bg-slate-grey-950 border border-indigo-smoke-800 px-3 py-2.5 flex flex-col gap-2">
            <div className="flex items-center justify-between gap-2">
              <span className="font-mono text-[10px] text-parchment-300 truncate flex-1 min-w-0">
                {activeDownload.filename}
              </span>
              <button
                onClick={onCancelDownload}
                className="flex-shrink-0 text-slate-grey-600 hover:text-brick-red-400 transition-colors"
                title="Cancel download"
              >
                <X size={11} />
              </button>
            </div>

            <div className="w-full h-1 bg-slate-grey-800 rounded-full overflow-hidden">
              <div
                className="h-full bg-indigo-smoke-500 rounded-full transition-[width] duration-75"
                style={{ width: `${activeDownload.percent.toFixed(1)}%` }}
              />
            </div>

            <div className="flex items-center justify-between">
              <span className="font-mono text-[10px] text-slate-grey-500 tabular-nums">
                {formatBytes(activeDownload.downloaded)}
                {activeDownload.total > 0 &&
                  ` / ${formatBytes(activeDownload.total)}`}
              </span>
              <span className="font-mono text-[10px] text-indigo-smoke-400 tabular-nums">
                {activeDownload.percent.toFixed(1)}%
              </span>
            </div>

            <p className="font-mono text-[10px] text-slate-grey-600 truncate">
              {activeDownload.model_id}
            </p>
          </div>
        </div>
      )}

      {/* ── Loaded model status ─────────────────────────────────────────── */}
      <div>
        <SectionHead label="Active Model" />

        {/* Loading state */}
        {isLoading && (
          <div className="rounded-md bg-slate-grey-950 border border-indigo-smoke-800/50 px-3 py-2.5 flex items-center gap-2">
            <Loader2
              size={11}
              className="text-indigo-smoke-400 animate-spin flex-shrink-0"
            />
            <span className="font-mono text-[10px] text-indigo-smoke-400">
              {loadingState.backend
                ? `loading on ${loadingState.backend}…`
                : "loading model…"}
            </span>
          </div>
        )}

        {/* Loaded model card */}
        {loadedModel && !isLoading && (
          <div className="rounded-md bg-slate-grey-950 border border-emerald-800/40 px-3 py-2.5 flex flex-col gap-1.5">
            <div className="flex items-start justify-between gap-1.5">
              <span className="font-mono text-xs text-parchment-200 break-all leading-tight flex-1">
                {loadedModel.filename}
              </span>
              <button
                onClick={handleUnloadModel}
                className="flex-shrink-0 mt-0.5 text-slate-grey-600 hover:text-brick-red-400 transition-colors"
                title="Unload model"
              >
                <Square size={11} />
              </button>
            </div>
            <div className="flex items-center gap-1.5 flex-wrap">
              <BackendBadge backend={loadedModel.backend} />
              <span className="font-mono text-[10px] text-slate-grey-600 truncate">
                {loadedModel.model_id}
              </span>
            </div>
          </div>
        )}

        {/* No model loaded and not loading */}
        {!loadedModel && !isLoading && (
          <div className="rounded-md bg-slate-grey-950 border border-slate-grey-800 px-3 py-2.5">
            <p className="font-body text-xs text-slate-grey-600 italic">
              No model loaded. Click ▶ on a downloaded model below.
            </p>
          </div>
        )}
      </div>

      {/* ── Downloaded models ───────────────────────────────────────────── */}
      <div>
        <SectionHead label="Models" />

        {downloadedModels.length === 0 ? (
          <div className="rounded-md bg-slate-grey-950 border border-slate-grey-800 px-3 py-2.5">
            <p className="font-body text-xs text-slate-grey-600 italic">
              No models downloaded yet. Search and download one from the header.
            </p>
          </div>
        ) : (
          <div className="flex flex-col gap-1.5">
            {downloadedModels.map((m) => {
              const active = isLoaded(m.model_id, m.filename);
              return (
                <div
                  key={`${m.model_id}::${m.filename}`}
                  className={`rounded-md bg-slate-grey-950 border px-3 py-2 flex flex-col gap-1 group transition-colors ${
                    active
                      ? "border-emerald-800/40"
                      : "border-slate-grey-800 hover:border-slate-grey-700"
                  }`}
                >
                  <div className="flex items-start justify-between gap-1.5">
                    <span className="font-mono text-xs text-parchment-200 break-all leading-tight flex-1">
                      {m.filename}
                    </span>
                    <div className="flex items-center gap-1 flex-shrink-0 mt-0.5">
                      {/* Load button — hidden when this model is loaded or another is loading */}
                      {!active && (
                        <button
                          onClick={() =>
                            handleLoadModel(m.model_id, m.filename)
                          }
                          disabled={isLoading}
                          className="text-slate-grey-600 hover:text-emerald-400 transition-colors disabled:opacity-30 disabled:cursor-not-allowed opacity-0 group-hover:opacity-100"
                          title="Load model"
                        >
                          <Play size={11} />
                        </button>
                      )}
                      {/* Delete button */}
                      <button
                        onClick={() =>
                          handleDeleteModel(m.model_id, m.filename)
                        }
                        disabled={active || isLoading}
                        className="text-slate-grey-700 hover:text-brick-red-400 transition-colors disabled:opacity-30 disabled:cursor-not-allowed opacity-0 group-hover:opacity-100"
                        title={
                          active
                            ? "Unload the model before deleting"
                            : "Delete model"
                        }
                      >
                        <Trash2 size={11} />
                      </button>
                    </div>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <HardDrive
                      size={9}
                      className="text-slate-grey-600 flex-shrink-0"
                    />
                    <span className="font-mono text-[10px] text-slate-grey-500 tabular-nums">
                      {formatBytes(m.size)}
                    </span>
                    <span className="font-mono text-[10px] text-slate-grey-700 truncate flex-1 min-w-0">
                      · {m.model_id}
                    </span>
                    {active && loadedModel && (
                      <BackendBadge backend={loadedModel.backend} />
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* ── Creativity ──────────────────────────────────────────────────── */}
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

      {/* ── Hardware ────────────────────────────────────────────────────── */}
      <div>
        <SectionHead label="hardware" />
        <HardwarePanel />
      </div>
    </aside>
  );
}
