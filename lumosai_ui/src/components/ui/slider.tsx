import React from "react";

interface SliderProps {
  className?: string;
  value?: number[];
  defaultValue?: number[];
  min?: number;
  max?: number;
  step?: number;
  onValueChange?: (value: number[]) => void;
}

export function Slider({ 
  className, 
  value, 
  defaultValue = [0],
  min = 0,
  max = 100,
  step = 1,
  onValueChange 
}: SliderProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = [parseInt(e.target.value, 10)];
    if (onValueChange) {
      onValueChange(newValue);
    }
  };

  return (
    <div className={`slider-container ${className || ""}`}>
      <input 
        type="range"
        className="slider"
        min={min}
        max={max}
        step={step}
        value={value ? value[0] : defaultValue[0]}
        onChange={handleChange}
      />
    </div>
  );
} 