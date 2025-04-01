import React from "react";

interface ToastProps {
  title?: string;
  description?: string;
  variant?: "default" | "destructive";
}

interface ToastActionElement {
  altText: string;
}

export type ToastActionProps = React.ButtonHTMLAttributes<HTMLButtonElement>;

export function ToastAction(props: ToastActionProps): ToastActionElement {
  return { altText: props.children as string };
}

interface ToastApi {
  toast: (props: ToastProps) => void;
  dismiss: (toastId?: string) => void;
}

// Simple mock implementation of the useToast hook
export const useToast = (): ToastApi => {
  return {
    toast: (props: ToastProps) => {
      console.log("Toast:", props);
    },
    dismiss: (toastId?: string) => {
      console.log("Dismiss toast:", toastId);
    },
  };
}; 