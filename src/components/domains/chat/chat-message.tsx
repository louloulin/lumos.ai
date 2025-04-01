"use client";

import React from "react";
import { cn } from "@/lib/utils";
import { Message } from "./chat-container";

interface ChatMessageProps {
  message: Message;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === "user";
  
  return (
    <div
      className={cn(
        "flex flex-col space-y-2 p-4 rounded-lg",
        isUser
          ? "self-end bg-primary text-primary-foreground"
          : "self-start bg-muted"
      )}
    >
      <div className="text-sm font-medium">
        {isUser ? "You" : "Assistant"}
      </div>
      <div className="text-sm whitespace-pre-wrap">{message.content}</div>
      {message.timestamp && (
        <div className="text-xs opacity-70">
          {message.timestamp.toLocaleTimeString()}
        </div>
      )}
    </div>
  );
} 