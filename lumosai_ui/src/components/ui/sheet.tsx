import React from "react";

interface SheetProps {
  className?: string;
  children?: React.ReactNode;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  side?: "left" | "right" | "top" | "bottom";
}

export function Sheet({ className, children, open, onOpenChange }: SheetProps) {
  return (
    <div className={`sheet ${className || ""}`} style={{ display: open ? "block" : "none" }}>
      {children}
    </div>
  );
}

export function SheetTrigger({ className, children }: { className?: string; children?: React.ReactNode }) {
  return (
    <button className={`sheet-trigger ${className || ""}`}>
      {children}
    </button>
  );
}

export function SheetContent({ className, children }: { className?: string; children?: React.ReactNode }) {
  return (
    <div className={`sheet-content ${className || ""}`}>
      {children}
    </div>
  );
}

export function SheetHeader({ className, children }: { className?: string; children?: React.ReactNode }) {
  return (
    <div className={`sheet-header ${className || ""}`}>
      {children}
    </div>
  );
}

export function SheetFooter({ className, children }: { className?: string; children?: React.ReactNode }) {
  return (
    <div className={`sheet-footer ${className || ""}`}>
      {children}
    </div>
  );
}

export function SheetTitle({ className, children }: { className?: string; children?: React.ReactNode }) {
  return (
    <h3 className={`sheet-title ${className || ""}`}>
      {children}
    </h3>
  );
}

export function SheetDescription({ className, children }: { className?: string; children?: React.ReactNode }) {
  return (
    <p className={`sheet-description ${className || ""}`}>
      {children}
    </p>
  );
}

export function SheetClose({ className, children }: { className?: string; children?: React.ReactNode }) {
  return (
    <button className={`sheet-close ${className || ""}`}>
      {children}
    </button>
  );
} 