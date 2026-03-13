import { useState, useEffect, useRef } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { ChatMessage, CreativityKey, Session } from "./types";
import { Header, Sidebar } from "./components/layout";
import { ChatArea } from "./components/chat";
import { SettingsPanel } from "./components/settings";
import { startChat, stopChat, hasToken } from "./lib/Chat";
import "./index.css";

function createSession(model: string, title: string = "New session"): Session {
  return { id: Date.now(), title, model: model || "none", time: "now" };
}

export default function App() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<number | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [availableModels] = useState<string[]>([]);
  const [model, setModel] = useState("");
  const [creativity, setCreativity] = useState<CreativityKey>("balanced");
  const [showSettings, setShowSettings] = useState(false);
  const [isReady, setIsReady] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // Whether a token is saved in the Rust store — we never hold the value here
  const [tokenSaved, setTokenSaved] = useState(false);

  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    if (!isReady && sessions.length === 0) {
      const defaultSession = createSession("");
      setSessions([defaultSession]);
      setActiveSessionId(defaultSession.id);
      setIsReady(true);
    }
    hasToken()
      .then(setTokenSaved)
      .catch(() => setTokenSaved(false));
  }, [isReady, sessions.length]);

  const activeSession = sessions.find((s) => s.id === activeSessionId);
  if (!activeSession)
    return <div className="bg-slate-grey-950 h-screen w-screen" />;

  const handleSend = async () => {
    if (!input.trim() || isLoading) return;
    if (!model) {
      setError("Please select a model first.");
      return;
    }
    if (!tokenSaved) {
      setError("Please add your HuggingFace token in Settings.");
      return;
    }

    setError(null);

    const userMsg: ChatMessage = {
      id: Date.now(),
      role: "user",
      content: input.trim(),
    };
    const assistantId = Date.now() + 1;
    const assistantMsg: ChatMessage = {
      id: assistantId,
      role: "assistant",
      content: "",
    };

    setMessages((prev) => [...prev, userMsg, assistantMsg]);
    setInput("");
    setIsLoading(true);

    const history = [...messages, userMsg]
      .filter((m) => m.role !== "system")
      .map((m) => ({
        role: m.role as "user" | "assistant",
        content: m.content,
      }));

    unlistenRef.current = await startChat(model, history, {
      onToken: (chunk) => {
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantId ? { ...m, content: m.content + chunk } : m,
          ),
        );
      },
      onDone: () => {
        setIsLoading(false);
        unlistenRef.current = null;
      },
      onError: (msg) => {
        setError(msg);
        setIsLoading(false);
        setMessages((prev) => prev.filter((m) => m.id !== assistantId));
        unlistenRef.current = null;
      },
    });
  };

  const handleStop = async () => {
    await stopChat();
    unlistenRef.current?.();
    unlistenRef.current = null;
    setIsLoading(false);
  };

  const resetSession = () => {
    handleStop();
    setMessages([]);
    setError(null);
  };

  const handleNewSession = () => {
    resetSession();
    const s = createSession(model);
    setSessions((prev) => [s, ...prev]);
    setActiveSessionId(s.id);
  };

  const handleSelectSession = (id: number) => {
    resetSession();
    setActiveSessionId(id);
  };

  const handleDeleteSession = (id: number) => {
    setSessions((prev) => {
      const remaining = prev.filter((s) => s.id !== id);
      if (remaining.length === 0) {
        const replacement = createSession(model);
        setActiveSessionId(replacement.id);
        setMessages([]);
        return [replacement];
      }
      if (activeSessionId === id) {
        const idx = prev.findIndex((s) => s.id === id);
        setActiveSessionId(remaining[Math.min(idx, remaining.length - 1)].id);
        setMessages([]);
      }
      return remaining;
    });
  };

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-slate-grey-950 text-parchment-300">
      <Header model={model} onModelChange={setModel} />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar
          sessions={sessions}
          activeSessionId={activeSessionId}
          onSelectSession={handleSelectSession}
          onNewSession={handleNewSession}
          onDeleteSession={handleDeleteSession}
          showSettings={showSettings}
          onToggleSettings={() => setShowSettings((p) => !p)}
        />
        <ChatArea
          activeSession={activeSession}
          messages={messages}
          input={input}
          creativity={creativity}
          onInputChange={setInput}
          onSend={handleSend}
          onStop={handleStop}
          isLoading={isLoading}
          error={error}
        />
        {showSettings && (
          <SettingsPanel
            model={model}
            availableModels={availableModels}
            creativity={creativity}
            tokenSaved={tokenSaved}
            onModelChange={setModel}
            onCreativityChange={setCreativity}
            onTokenSaved={() => setTokenSaved(true)}
            onTokenDeleted={() => setTokenSaved(false)}
          />
        )}
      </div>
    </div>
  );
}
