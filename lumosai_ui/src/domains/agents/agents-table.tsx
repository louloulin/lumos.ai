'use client';

import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { MoreHorizontal, RefreshCw, Eye, Settings, Trash2 } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog';
import { LomusaiClient } from '@lomusai/client-js';
import { useToast } from '@/components/ui/use-toast';

interface AgentsTableProps {
  agents: any[];
  isLoading: boolean;
  onRefresh: () => void;
}

export function AgentsTable({ agents, isLoading, onRefresh }: AgentsTableProps) {
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [agentToDelete, setAgentToDelete] = useState<any>(null);
  const { toast } = useToast();

  const handleDeleteClick = (agent: any) => {
    setAgentToDelete(agent);
    setDeleteDialogOpen(true);
  };

  const handleDeleteConfirm = async () => {
    if (!agentToDelete) return;

    try {
      const client = new LomusaiClient({
        baseUrl: import.meta.env.VITE_API_BASE_URL || '',
      });
      
      await client.deleteAgent(agentToDelete.id);
      
      toast({
        title: 'Agent deleted',
        description: `${agentToDelete.name} has been deleted successfully`,
      });
      
      setDeleteDialogOpen(false);
      setAgentToDelete(null);
      onRefresh();
    } catch (error) {
      console.error('Error deleting agent:', error);
      toast({
        title: 'Error deleting agent',
        description: 'Could not delete the agent',
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
              <TableHead>Model</TableHead>
              <TableHead>Created At</TableHead>
              <TableHead className="w-[100px]">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8">
                  Loading agents...
                </TableCell>
              </TableRow>
            ) : agents.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center py-8">
                  No agents found. Create one to get started.
                </TableCell>
              </TableRow>
            ) : (
              agents.map((agent) => (
                <TableRow key={agent.id}>
                  <TableCell className="font-medium">{agent.name}</TableCell>
                  <TableCell className="max-w-[300px] truncate">{agent.description || 'No description'}</TableCell>
                  <TableCell>{agent.model || 'Default'}</TableCell>
                  <TableCell>{new Date(agent.createdAt).toLocaleString()}</TableCell>
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
                          <Link to={`/agents/${agent.id}/chat`}>
                            <Eye className="h-4 w-4 mr-2" />
                            Chat
                          </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem asChild>
                          <Link to={`/agents/${agent.id}/settings`}>
                            <Settings className="h-4 w-4 mr-2" />
                            Settings
                          </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleDeleteClick(agent)}>
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
              Are you sure you want to delete the agent "{agentToDelete?.name}"? This action cannot be undone.
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
