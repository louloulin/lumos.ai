import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { Toast, ToastProvider, ToastTitle, ToastDescription, ToastViewport } from '../toast';
import { useToast } from '../use-toast';
import { Toaster } from '../toaster';

// Create a test component that uses the toast hook
const TestComponent = ({ show = false, variant = 'default' }: { show?: boolean, variant?: 'default' | 'destructive' }) => {
  const { toast } = useToast();
  
  if (show) {
    toast({
      variant,
      title: 'Test Toast Title',
      description: 'Test Toast Description',
    });
  }
  
  return <Toaster />;
};

// Helper component for testing the Toast component directly
const SimpleToast = () => (
  <ToastProvider swipeDirection="right">
    <Toast>
      <ToastTitle>Test Title</ToastTitle>
      <ToastDescription>Test Description</ToastDescription>
    </Toast>
    <ToastViewport />
  </ToastProvider>
);

describe('Toast', () => {
  beforeEach(() => {
    // Reset any toasts that might be lingering from previous tests
    vi.resetModules();
  });

  it('renders toast with correct content', async () => {
    render(<SimpleToast />);
    
    // We need to wait for the Toast to be rendered in the portal
    await waitFor(() => {
      expect(screen.getByText('Test Title')).toBeInTheDocument();
      expect(screen.getByText('Test Description')).toBeInTheDocument();
    });
  });

  it('displays toast via useToast hook', async () => {
    render(<TestComponent show={true} />);
    
    // We need to wait for the Toast to be rendered in the portal
    await waitFor(() => {
      expect(screen.getByText('Test Toast Title')).toBeInTheDocument();
      expect(screen.getByText('Test Toast Description')).toBeInTheDocument();
    });
  });

  it('applies correct variant class', async () => {
    render(
      <ToastProvider swipeDirection="right">
        <Toast variant="destructive">
          <ToastTitle>Destructive Toast</ToastTitle>
        </Toast>
        <ToastViewport />
      </ToastProvider>
    );

    // We need to wait for the Toast to be rendered in the portal
    await waitFor(() => {
      const title = screen.getByText('Destructive Toast');
      expect(title).toBeInTheDocument();
      
      // Find the element with role="status" which should have the destructive class
      const toastElement = screen.getByRole('status');
      expect(toastElement).toHaveClass('destructive');
    });
  });
}); 