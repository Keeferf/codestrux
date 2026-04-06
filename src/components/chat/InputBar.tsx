import { LuSend, LuSquare, LuPaperclip } from "react-icons/lu";
import { useRef } from "react";
import {
  FileAttachment,
  AttachedFile,
  FileAttachmentRef,
} from "./FileAttachment";

interface InputBarProps {
  input: string;
  canSend: boolean;
  isLoading: boolean;
  attachedFiles: AttachedFile[];
  onInputChange: (value: string) => void;
  onSend: () => void | Promise<void>;
  onStop: () => void | Promise<void>;
  onFilesAttach: (files: AttachedFile[]) => void;
  onFileRemove: (fileId: string) => void;
}

export function InputBar({
  input,
  canSend,
  isLoading,
  attachedFiles,
  onInputChange,
  onSend,
  onStop,
  onFilesAttach,
  onFileRemove,
}: InputBarProps) {
  const fileAttachRef = useRef<FileAttachmentRef | null>(null);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (canSend && !isLoading) onSend();
    }
  };

  return (
    <div className="shrink-0 px-4.5 py-2 bg-slate-grey-900 border-t border-slate-grey-800">
      <div className="flex flex-col gap-2 bg-slate-grey-950 border border-slate-grey-800 rounded-lg p-2">
        {/* File previews (above input) */}
        <FileAttachment
          ref={fileAttachRef}
          onFilesAttach={onFilesAttach}
          onFileRemove={onFileRemove}
          attachedFiles={attachedFiles}
          disabled={isLoading}
        />

        {/* Input row */}
        <div className="flex gap-2 items-start">
          {/* Paperclip trigger */}
          <button
            onClick={() => fileAttachRef.current?.openFilePicker()}
            disabled={isLoading}
            className="flex items-center justify-center mt-1 p-1 rounded-md text-slate-grey-500 hover:text-parchment-300 hover:bg-slate-grey-800 transition-all duration-200 disabled:opacity-40 disabled:cursor-default"
            aria-label="Attach files"
          >
            <LuPaperclip size={15} />
          </button>

          {/* Text input */}
          <textarea
            value={input}
            onChange={(e) => onInputChange(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={isLoading ? "Generating…" : "Message…"}
            disabled={isLoading}
            rows={1}
            className="flex-1 bg-transparent border-none outline-none resize-none font-body text-sm leading-relaxed min-h-5 text-parchment-200 placeholder:text-slate-grey-500 py-1 disabled:opacity-40"
          />

          {/* Actions */}
          <div className="flex gap-2 items-center">
            {isLoading ? (
              <button
                onClick={onStop}
                className="flex items-center gap-1.5 rounded-md px-3 py-2 font-display text-xs border transition-all duration-200 bg-brick-red-900/40 border-brick-red-800 text-brick-red-400 hover:bg-brick-red-800/60 hover:border-brick-red-700"
              >
                <LuSquare size={11} fill="currentColor" />
                stop
              </button>
            ) : (
              <button
                onClick={onSend}
                disabled={!canSend}
                className={`flex items-center gap-1.5 rounded-md px-3 py-2 font-display text-xs border transition-all duration-200 ${
                  canSend
                    ? "bg-indigo-smoke-700 border-indigo-smoke-600 text-parchment-100 hover:bg-indigo-smoke-600 hover:border-indigo-smoke-500"
                    : "bg-transparent border-slate-grey-800 text-slate-grey-600 cursor-default"
                }`}
              >
                <LuSend size={13} />
                send
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
