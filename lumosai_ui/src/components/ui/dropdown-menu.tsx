import React from "react";

interface DropdownMenuProps {
  className?: string;
  children?: React.ReactNode;
}

export function DropdownMenu({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuTrigger({ className, children }: DropdownMenuProps) {
  return (
    <button className={`dropdown-menu-trigger ${className || ""}`}>
      {children}
    </button>
  );
}

export function DropdownMenuContent({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-content ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuItem({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-item ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuSeparator({ className }: DropdownMenuProps) {
  return (
    <hr className={`dropdown-menu-separator ${className || ""}`} />
  );
}

export function DropdownMenuLabel({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-label ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuGroup({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-group ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuSub({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-sub ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuSubTrigger({ className, children }: DropdownMenuProps) {
  return (
    <button className={`dropdown-menu-sub-trigger ${className || ""}`}>
      {children}
    </button>
  );
}

export function DropdownMenuSubContent({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-sub-content ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuCheckboxItem({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-checkbox-item ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuRadioGroup({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-radio-group ${className || ""}`}>
      {children}
    </div>
  );
}

export function DropdownMenuRadioItem({ className, children }: DropdownMenuProps) {
  return (
    <div className={`dropdown-menu-radio-item ${className || ""}`}>
      {children}
    </div>
  );
} 