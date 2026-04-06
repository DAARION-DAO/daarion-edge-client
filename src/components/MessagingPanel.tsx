import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { MessageSquare, Users, Shield, Zap } from "lucide-react";

interface Message {
  id: string;
  sender: string;
  content: string;
  timestamp: number;
}

interface RoomInfo {
  room_id: string;
  display_name: string;
  participants: string[];
}

interface MessagingSession {
  session_id: string;
  messaging_token: string;
}

type ConnectivityState = 
  | "Disconnected" 
  | "Connecting" 
  | "Connected" 
  | { Reconnecting: { attempt: number, next_retry_sec: number } }
  | { Error: string };

export function MessagingPanel() {
  const [connectivity, setConnectivity] = useState<ConnectivityState>("Disconnected");
  const [roomInfo, setRoomInfo] = useState<RoomInfo | null>(null);
  const [session, setSession] = useState<MessagingSession | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);

  async function init() {
    try {
      const [status, msgs] = await Promise.all([
        invoke<[ConnectivityState, RoomInfo | null, MessagingSession | null]>("get_messaging_status"),
        invoke<Message[]>("get_node_messages")
      ]);
      setConnectivity(status[0]);
      setRoomInfo(status[1]);
      setSession(status[2]);
      setMessages(msgs);
    } catch (e) {
      console.error("Failed to init messaging:", e);
    }
  }

  async function bootstrap() {
    try {
      await invoke("bootstrap_messaging");
    } catch (e) {
      console.error("Bootstrap failed:", e);
    }
  }



  useEffect(() => {
    init();

    const unlistenStatus = listen<ConnectivityState>("messaging-status-changed", (event) => {
      setConnectivity(event.payload);
    });

    const unlistenRoom = listen<RoomInfo>("messaging-room-ready", (event) => {
      setRoomInfo(event.payload);
    });

    const unlistenMsg = listen<Message>("messaging-new-message", (event) => {
      setMessages(prev => [...prev, event.payload]);
    });

    return () => {
      unlistenStatus.then(f => f());
      unlistenRoom.then(f => f());
      unlistenMsg.then(f => f());
    };
  }, []);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  const isError = typeof connectivity === "object" && "Error" in connectivity;
  const errorMsg = isError ? (connectivity as any).Error : "";

  const isReconnecting = typeof connectivity === "object" && "Reconnecting" in connectivity;
  const reconnectInfo = isReconnecting ? (connectivity as any).Reconnecting : null;

  return (
    <div className="flex flex-col h-[520px] glass border-white/5 overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b border-white/5 flex items-center justify-between bg-white/[0.02]">
        <div className="flex items-center gap-3">
          <div className={`p-2 rounded-lg ${connectivity === "Connected" ? 'bg-blue-500/10 text-blue-400' : 'bg-white/5 text-white/20'}`}>
            <MessageSquare size={16} />
          </div>
          <div>
            <h2 className="text-[10px] font-bold uppercase tracking-[0.2em] text-white/70">Node Control Room</h2>
            <p className="text-[9px] text-white/30 font-mono italic">{roomInfo?.room_id || "Awaiting Bootstrap"}</p>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          {connectivity === "Connected" ? (
             <div className="flex items-center gap-2 px-2 py-1 bg-emerald-500/10 border border-emerald-500/20 rounded text-[9px] text-emerald-400 font-bold uppercase">
               <Zap size={10} /> Active
             </div>
          ) : isReconnecting ? (
             <div className="flex flex-col items-end">
               <div className="text-[9px] text-blue-400 font-bold uppercase animate-pulse">Reconnecting... (Attempt {reconnectInfo.attempt})</div>
               <div className="text-[8px] text-white/20">Next try in {reconnectInfo.next_retry_sec}s</div>
             </div>
          ) : connectivity === "Connecting" ? (
             <div className="text-[9px] text-blue-400 font-bold uppercase animate-pulse">Syncing...</div>
          ) : (
             <button onClick={bootstrap} className="btn-primary py-1 px-3 text-[10px]">Bootstrap</button>
          )}
        </div>
      </div>

      {/* Session Info (M1.5) */}
      {session && (
        <div className="px-4 py-1.5 bg-blue-500/5 border-b border-white/5 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Shield size={10} className="text-blue-400/50" />
              <span className="text-[8px] text-white/30 uppercase font-black tracking-widest">Active Session:</span>
              <code className="text-[9px] text-blue-300/40 font-mono">{session.session_id}</code>
            </div>
            <div className="text-[8px] text-white/10 uppercase font-bold">Encrypted Channel</div>
        </div>
      )}

      {/* Participants Bar */}
      {roomInfo && (
        <div className="px-4 py-2 bg-white/[0.01] border-b border-white/5 flex items-center gap-4">
          <div className="flex items-center gap-1.5 text-white/30">
            <Users size={12} />
            <span className="text-[9px] font-bold uppercase tracking-wider">Agents:</span>
          </div>
          <div className="flex gap-2">
            {roomInfo.participants.map(p => (
              <span key={p} className="text-[9px] px-1.5 py-0.5 bg-white/5 rounded text-white/60 font-medium">@{p}</span>
            ))}
          </div>
        </div>
      )}

      {/* Messages Area / Sovereign Matrix UI */}
      <div className="flex-1 flex flex-col relative bg-[#181b21]">
        {connectivity !== "Connected" ? (
          <div className="h-full flex flex-col items-center justify-center opacity-20 py-10 text-center">
            <Shield size={32} className="mb-2" />
            <p className="text-[10px] font-bold uppercase tracking-widest leading-relaxed">
              Establish Control Plane connection to view Sovereign Chat
            </p>
          </div>
        ) : (
          <iframe 
            src="https://chat.daarion.space" 
            className="w-full h-full border-none"
            title="DAARION Sovereign Matrix"
            allow="camera; microphone; display-capture; autoplay; clipboard-write; clipboard-read"
          />
        )}
        
        {isError && (
          <div className="absolute bottom-4 left-4 right-4 p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-[10px] text-red-400 font-medium italic backdrop-blur-md">
            Connection Error: {errorMsg}
          </div>
        )}
      </div>
    </div>
  );
}
