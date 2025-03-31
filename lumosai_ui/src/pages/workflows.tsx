import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { PlusIcon } from 'lucide-react';
import { WorkflowsTable } from '@/domains/workflows/workflows-table';
import { LomusaiClient } from '@lomusai/client-js';
import { useToast } from '@/components/ui/use-toast';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';

export default function WorkflowsPage() {
  const [workflows, setWorkflows] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const { toast } = useToast();

  // New workflow form state
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [newWorkflowName, setNewWorkflowName] = useState('');
  const [newWorkflowDescription, setNewWorkflowDescription] = useState('');

  const client = new LomusaiClient({
    baseUrl: import.meta.env.VITE_API_BASE_URL || '',
  });

  const fetchWorkflows = async () => {
    setIsLoading(true);
    try {
      const response = await client.getWorkflows();
      setWorkflows(response);
    } catch (error) {
      console.error('Error fetching workflows:', error);
      toast({
        title: 'Error fetching workflows',
        description: 'Could not retrieve the list of workflows',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateWorkflow = async () => {
    if (!newWorkflowName.trim()) {
      toast({
        title: 'Validation error',
        description: 'Workflow name is required',
        variant: 'destructive',
      });
      return;
    }

    try {
      await client.createWorkflow({
        name: newWorkflowName,
        description: newWorkflowDescription,
        steps: [], // Initially empty workflow
      });
      
      toast({
        title: 'Workflow created',
        description: `${newWorkflowName} has been created successfully`,
      });
      
      // Reset form and close dialog
      setNewWorkflowName('');
      setNewWorkflowDescription('');
      setIsCreateDialogOpen(false);
      
      // Refresh workflow list
      fetchWorkflows();
    } catch (error) {
      console.error('Error creating workflow:', error);
      toast({
        title: 'Error creating workflow',
        description: 'Could not create the workflow',
        variant: 'destructive',
      });
    }
  };

  useEffect(() => {
    fetchWorkflows();
  }, []);

  return (
    <div className="container mx-auto py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Lumos Workflows</h1>
        
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <PlusIcon className="mr-2 h-4 w-4" />
              Create Workflow
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[425px]">
            <DialogHeader>
              <DialogTitle>Create New Workflow</DialogTitle>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="name">Name</Label>
                <Input 
                  id="name" 
                  value={newWorkflowName} 
                  onChange={(e) => setNewWorkflowName(e.target.value)}
                  placeholder="My Workflow" 
                />
              </div>
              
              <div className="grid gap-2">
                <Label htmlFor="description">Description</Label>
                <Textarea 
                  id="description" 
                  value={newWorkflowDescription} 
                  onChange={(e) => setNewWorkflowDescription(e.target.value)}
                  placeholder="Workflow description" 
                  rows={4}
                />
              </div>
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={handleCreateWorkflow}>
                Create
              </Button>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      <WorkflowsTable workflows={workflows} isLoading={isLoading} onRefresh={fetchWorkflows} />
    </div>
  );
} 