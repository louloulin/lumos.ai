'use client';

import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { MoreHorizontal, RefreshCw, Eye, Edit, Play, Trash2 } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog';
import { LomusaiClient } from '@lomusai/client-js';
import { useToast } from '@/components/ui/use-toast';

interface WorkflowsTableProps {
  workflows: any[];
  isLoading: boolean;
  onRefresh: () => void;
}

export function WorkflowsTable({ workflows, isLoading, onRefresh }: WorkflowsTableProps) {
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [workflowToDelete, setWorkflowToDelete] = useState<any>(null);
  const { toast } = useToast();

  const handleDeleteClick = (workflow: any) => {
    setWorkflowToDelete(workflow);
    setDeleteDialogOpen(true);
  };

  const handleDeleteConfirm = async () => {
    if (!workflowToDelete) return;

    try {
      const client = new LomusaiClient({
        baseUrl: import.meta.env.VITE_API_BASE_URL || '',
      });
      
      await client.deleteWorkflow(workflowToDelete.id);
      
      toast({
        title: 'Workflow deleted',
        description: `${workflowToDelete.name} has been deleted successfully`,
      });
      
      setDeleteDialogOpen(false);
      setWorkflowToDelete(null);
      onRefresh();
    } catch (error) {
      console.error('Error deleting workflow:', error);
      toast({
        title: 'Error deleting workflow',
        description: 'Could not delete the workflow',
        variant: 'destructive',
      });
    }
  };

  const handleRunWorkflow = async (workflowId: string) => {
    try {
      const client = new LomusaiClient({
        baseUrl: import.meta.env.VITE_API_BASE_URL || '',
      });
      
      await client.runWorkflow(workflowId);
      
      toast({
        title: 'Workflow started',
        description: 'The workflow has been started successfully',
      });
    } catch (error) {
      console.error('Error running workflow:', error);
      toast({
        title: 'Error running workflow',
        description: 'Could not run the workflow',
        variant: 'destructive',
      });
    }
  };

  return (
    <div>
      <div className="flex justify-end mb-4">
        <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
          <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Description</TableHead>
              <TableHead>Steps</TableHead>
              <TableHead>Created At</TableHead>
              <TableHead className="w-[100px]">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8">
                  Loading workflows...
                </TableCell>
              </TableRow>
            ) : workflows.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8">
                  No workflows found. Create one to get started.
                </TableCell>
              </TableRow>
            ) : (
              workflows.map((workflow) => (
                <TableRow key={workflow.id}>
                  <TableCell className="font-medium">{workflow.name}</TableCell>
                  <TableCell className="max-w-[300px] truncate">{workflow.description || 'No description'}</TableCell>
                  <TableCell>{workflow.steps?.length || 0} steps</TableCell>
                  <TableCell>{new Date(workflow.createdAt).toLocaleString()}</TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="icon">
                          <MoreHorizontal className="h-4 w-4" />
                          <span className="sr-only">Open menu</span>
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem asChild>
                          <Link to={`/workflows/${workflow.id}`}>
                            <Eye className="h-4 w-4 mr-2" />
                            View
                          </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem asChild>
                          <Link to={`/workflows/${workflow.id}/edit`}>
                            <Edit className="h-4 w-4 mr-2" />
                            Edit
                          </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleRunWorkflow(workflow.id)}>
                          <Play className="h-4 w-4 mr-2" />
                          Run
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleDeleteClick(workflow)}>
                          <Trash2 className="h-4 w-4 mr-2 text-destructive" />
                          <span className="text-destructive">Delete</span>
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm Deletion</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete the workflow "{workflowToDelete?.name}"? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleDeleteConfirm}>
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
