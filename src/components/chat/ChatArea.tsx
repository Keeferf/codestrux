import type { ChatMessage, CreativityKey, Session } from "../../types";
import { MessageList } from "./MessageList";
import { InputBar } from "./InputBar";

interface ChatAreaProps {
  activeSession: Session;
  messages: ChatMessage[];
  input: string;
  creativity: CreativityKey;
  onInputChange: (value: string) => void;
  onSend: () => void;
}

export function ChatArea({
  activeSession,
  messages,
  input,
  onInputChange,
  onSend,
}: ChatAreaProps) {
  const canSend = input.trim().length > 0;
  const nonSystemCount = messages.filter((m) => m.role !== "system").length;

  return (
    <main className="flex-1 flex flex-col overflow-hidden min-w-0">
      <div className="shrink-0 flex items-center gap-2.5 px-4.5 py-4.5 bg-slate-grey-900 border-b border-slate-grey-800">
        <span className="font-body text-sm font-semibold text-parchment-200">
          {activeSession.title}
        </span>
        <span className="text-[10px] text-slate-grey-500">·</span>
        <span className="font-body text-xs text-slate-grey-500">
          {nonSystemCount} messages
        </span>
      </div>

      <MessageList messages={messages} />
      <InputBar
        input={input}
        canSend={canSend}
        onInputChange={onInputChange}
        onSend={onSend}
      />
    </main>
  );
}
