import { useEffect, useRef, useState, useCallback } from "react";
import { LuArrowDown } from "react-icons/lu";
import type { ChatMessage } from "../../types";
import { Message } from "./Message";

interface MessageListProps {
  messages: ChatMessage[];
  isLoading: boolean;
  onScrollbarWidth?: (width: number) => void;
}

export function MessageList({
  messages,
  isLoading,
  onScrollbarWidth,
}: MessageListProps) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [showScrollButton, setShowScrollButton] = useState(false);

  const scrollToBottom = useCallback(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  // Auto-scroll on new messages only if already near bottom
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    if (distanceFromBottom < 100) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages]);

  // Track scroll position to show/hide button
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const handleScroll = () => {
      const distanceFromBottom =
        el.scrollHeight - el.scrollTop - el.clientHeight;
      setShowScrollButton(distanceFromBottom > 100);
    };
    el.addEventListener("scroll", handleScroll, { passive: true });
    return () => el.removeEventListener("scroll", handleScroll);
  }, []);

  useEffect(() => {
    if (!containerRef.current || !onScrollbarWidth) return;
    const el = containerRef.current;
    const measure = () => onScrollbarWidth(el.offsetWidth - el.clientWidth);
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, [onScrollbarWidth]);

  return (
    <div className="relative flex-1 overflow-hidden">
      <div
        ref={containerRef}
        className="h-full overflow-y-auto px-40 pt-2.5 pb-24 flex flex-col"
        style={{ scrollbarGutter: "stable" }}
      >
        {messages.length === 0 ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <div className="font-display text-sm uppercase tracking-wider text-warm-grey-600 mb-2">
                No messages yet
              </div>
              <p className="font-display text-xs text-warm-grey-600">
                Type a message below to start the conversation
              </p>
            </div>
          </div>
        ) : (
          <>
            {messages.map((m) => (
              <Message key={m.id} message={m} />
            ))}
            <div className="pt-2.5 pb-5">
              <div className="inline-flex items-center gap-1.5 font-body text-xs text-warm-grey-600">
                {isLoading ? (
                  <>
                    <span className="inline-flex gap-0.75">
                      <span className="w-1.25 h-1.25 rounded-full bg-moss-green-500 animate-pulse [animation-delay:0s]" />
                      <span className="w-1.25 h-1.25 rounded-full bg-moss-green-500 animate-pulse [animation-delay:0.2s]" />
                      <span className="w-1.25 h-1.25 rounded-full bg-moss-green-500 animate-pulse [animation-delay:0.4s]" />
                    </span>
                    generating…
                  </>
                ) : (
                  <>
                    <span className="inline-flex gap-0.75">
                      <span className="w-1.25 h-1.25 rounded-full bg-indigo-smoke-500 animate-pulse [animation-delay:0s]" />
                      <span className="w-1.25 h-1.25 rounded-full bg-indigo-smoke-500 animate-pulse [animation-delay:0.2s]" />
                      <span className="w-1.25 h-1.25 rounded-full bg-indigo-smoke-500 animate-pulse [animation-delay:0.4s]" />
                    </span>
                    awaiting input
                  </>
                )}
              </div>
            </div>
          </>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Scroll-to-bottom button */}
      <div
        className={`absolute bottom-28 left-1/2 -translate-x-1/2 transition-all duration-200 ${
          showScrollButton
            ? "opacity-100 translate-y-0 pointer-events-auto"
            : "opacity-0 translate-y-2 pointer-events-none"
        }`}
      >
        <button
          onClick={scrollToBottom}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-slate-grey-800 border border-slate-grey-700 text-slate-grey-300 hover:bg-slate-grey-700 hover:text-parchment-100 hover:border-slate-grey-600 transition-all duration-150 shadow-lg shadow-black/30 font-body text-xs cursor-pointer"
          aria-label="Scroll to bottom"
        >
          <LuArrowDown size={12} />
          Scroll to bottom
        </button>
      </div>
    </div>
  );
}
