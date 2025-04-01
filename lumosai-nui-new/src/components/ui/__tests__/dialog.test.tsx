import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Dialog, DialogTrigger, DialogContent, DialogTitle, DialogDescription } from '../dialog';

describe('Dialog', () => {
  it('should open dialog when trigger is clicked', async () => {
    render(
      <Dialog>
        <DialogTrigger>Open Dialog</DialogTrigger>
        <DialogContent>
          <DialogTitle>Dialog Title</DialogTitle>
          <DialogDescription>Dialog Description</DialogDescription>
          <div>Dialog Content</div>
        </DialogContent>
      </Dialog>
    );

    // The dialog should be closed initially
    expect(screen.queryByText('Dialog Title')).not.toBeInTheDocument();
    expect(screen.queryByText('Dialog Description')).not.toBeInTheDocument();
    expect(screen.queryByText('Dialog Content')).not.toBeInTheDocument();

    // Click the trigger button
    const trigger = screen.getByText('Open Dialog');
    fireEvent.click(trigger);

    // Now the dialog should be visible
    expect(screen.getByText('Dialog Title')).toBeInTheDocument();
    expect(screen.getByText('Dialog Description')).toBeInTheDocument();
    expect(screen.getByText('Dialog Content')).toBeInTheDocument();
  });

  it('should close dialog when close button is clicked', async () => {
    render(
      <Dialog>
        <DialogTrigger>Open Dialog</DialogTrigger>
        <DialogContent>
          <DialogTitle>Dialog Title</DialogTitle>
          <DialogDescription>Dialog Description</DialogDescription>
          <div>Dialog Content</div>
        </DialogContent>
      </Dialog>
    );

    // Open the dialog
    const trigger = screen.getByText('Open Dialog');
    fireEvent.click(trigger);

    // Find and click the close button
    // The close button has an X icon and a span with class="sr-only" containing "Close"
    const closeButton = screen.getByRole('button', { 
      name: "Close"
    });
    fireEvent.click(closeButton);

    // The dialog should be closed
    expect(screen.queryByText('Dialog Title')).not.toBeInTheDocument();
    expect(screen.queryByText('Dialog Description')).not.toBeInTheDocument();
    expect(screen.queryByText('Dialog Content')).not.toBeInTheDocument();
  });
}); 