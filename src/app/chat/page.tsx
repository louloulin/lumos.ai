"use client";

import { ChatContainer } from "@/components/domains/chat/chat-container";

export default function ChatPage() {
  return (
    <main className="container mx-auto py-10 h-screen flex flex-col">
      <h1 className="text-3xl font-bold mb-6">Chat Demo</h1>
      <div className="flex-1">
        <ChatContainer title="LumosAI Assistant" />
      </div>
    </main>
  );
} 