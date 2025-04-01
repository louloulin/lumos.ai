import React from "react";

interface CheckboxProps extends React.InputHTMLAttributes<HTMLInputElement> {
  className?: string;
  checked?: boolean;
  defaultChecked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
}

export function Checkbox({ 
  className, 
  checked, 
  defaultChecked,
  onCheckedChange,
  ...props
}: CheckboxProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (onCheckedChange) {
      onCheckedChange(e.target.checked);
    }
  };

  return (
    <input 
      type="checkbox"
      className={`checkbox ${className || ""}`}
      checked={checked}
      defaultChecked={defaultChecked}
      onChange={handleChange}
      {...props}
    />
  );
} 