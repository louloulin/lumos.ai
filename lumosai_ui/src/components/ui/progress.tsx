import React from "react";

interface ProgressProps {
  className?: string;
  value?: number;
  max?: number;
  /** The indicator value, from 0 to max */
  indicatorValue?: number;
}

export function Progress({ 
  className, 
  value = 0,
  max = 100,
  indicatorValue
}: ProgressProps) {
  const percentage = ((indicatorValue ?? value) / max) * 100;
  
  return (
    <div className={`progress-container ${className || ""}`}>
      <div className="progress-track">
        <div 
          className="progress-indicator" 
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
} 