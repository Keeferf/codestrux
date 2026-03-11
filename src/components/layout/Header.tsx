import { ChevronRight } from "lucide-react";

interface HeaderProps {
  model: string;
}

export function Header({ model }: HeaderProps) {
  const modelLoaded = model.length > 0;

  return (
    <header
      className="h-11 shrink-0 flex items-center justify-between px-4 bg-slate-grey-900 border-b border-slate-grey-800"
      style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
    >
      <div
        className="flex items-center gap-2.5"
        style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties}
      >
        <div className="w-5.5 h-5.5 rounded flex items-center justify-center font-display font-black text-sm text-parchment-100 bg-linear-to-br from-indigo-smoke-700 to-indigo-smoke-500">
          ∴
        </div>
        <span className="font-display text-sm font-semibold text-parchment-200 tracking-wide">
          CodeStrux
        </span>
      </div>

      <div
        className="flex items-center gap-2 bg-slate-grey-950 border border-slate-grey-800 rounded-md px-3 py-1 cursor-pointer hover:border-slate-grey-700 transition-colors"
        style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties}
      >
        <div
          className={`w-1.75 h-1.75 rounded-full transition-all duration-300 ${
            modelLoaded
              ? "bg-moss-green-500 shadow-[0_0_6px_rgba(115,155,115,0.6)]"
              : "bg-slate-grey-700"
          }`}
        />
        <span
          className={`font-mono text-xs ${modelLoaded ? "text-parchment-300" : "text-slate-grey-500"}`}
        >
          {modelLoaded ? model : "no model loaded"}
        </span>
        <ChevronRight size={13} className="text-slate-grey-500" />
      </div>

      <div style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties} />
    </header>
  );
}
