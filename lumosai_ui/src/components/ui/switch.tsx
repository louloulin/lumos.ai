'use client';

import React from "react";

interface SwitchProps {
  className?: string;
  checked?: boolean;
  defaultChecked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
}

export function Switch({ 
  className, 
  checked,
  defaultChecked,
  onCheckedChange,
  ...props
}: SwitchProps & React.HTMLAttributes<HTMLButtonElement>) {
  const [isChecked, setIsChecked] = React.useState(defaultChecked || false);
  
  React.useEffect(() => {
    if (checked !== undefined) {
      setIsChecked(checked);
    }
  }, [checked]);
  
  const handleToggle = () => {
    const newValue = !isChecked;
    setIsChecked(newValue);
    if (onCheckedChange) {
      onCheckedChange(newValue);
    }
  };

  return (
    <button
      type="button" 
      role="switch"
      aria-checked={isChecked}
      className={`switch ${isChecked ? 'switch-checked' : 'switch-unchecked'} ${className || ""}`}
      onClick={handleToggle}
      {...props}
    >
      <span className="switch-thumb" style={{ 
        transform: isChecked ? 'translateX(100%)' : 'translateX(0%)',
        marginLeft: isChecked ? '2px' : '2px'
      }} />
    </button>
  );
}
