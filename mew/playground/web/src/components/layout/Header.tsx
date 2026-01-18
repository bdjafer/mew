import { useSessionStore, useUIStore } from '../../stores';

export function Header() {
  const sessionId = useSessionStore((state) => state.sessionId);
  const toggleChat = useUIStore((state) => state.toggleChat);

  return (
    <header id="header">
      <h1>MEW Playground</h1>
      <div id="session-info">
        {sessionId !== null ? `Session: ${sessionId}` : 'No session'}
      </div>
      <button className="header-btn" onClick={toggleChat}>
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
        </svg>
        Chat
      </button>
    </header>
  );
}
