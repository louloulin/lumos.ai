import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { PlusIcon } from 'lucide-react';
import { AgentsTable } from '@/domains/agents/agents-table';
import { LomusaiClient } from '@lomusai/client-js';
import { useToast } from '@/components/ui/use-toast';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

export default function AgentsPage() {
  const [agents, setAgents] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [modelOptions, setModelOptions] = useState<string[]>([]);
  const { toast } = useToast();

  // New agent form state
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [newAgentName, setNewAgentName] = useState('');
  const [newAgentDescription, setNewAgentDescription] = useState('');
  const [newAgentModel, setNewAgentModel] = useState('');
  const [newAgentInstructions, setNewAgentInstructions] = useState('');

  const client = new LomusaiClient({
    baseUrl: import.meta.env.VITE_API_BASE_URL || '',
  });

  const fetchAgents = async () => {
    setIsLoading(true);
    try {
      const response = await client.getAgents();
      setAgents(response);
    } catch (error) {
      console.error('Error fetching agents:', error);
      toast({
        title: 'Error fetching agents',
        description: 'Could not retrieve the list of agents',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const fetchModels = async () => {
    try {
      // This will need to be implemented in the Lumos backend
      const response = await client.getModels();
      setModelOptions(response.map(model => model.id));
    } catch (error) {
      console.error('Error fetching models:', error);
      // Fallback to some default models if API isn't available yet
      setModelOptions(['gpt-4-turbo', 'gpt-3.5-turbo', 'claude-3-opus']);
    }
  };

  const handleCreateAgent = async () => {
    if (!newAgentName.trim()) {
      toast({
        title: 'Validation error',
        description: 'Agent name is required',
        variant: 'destructive',
      });
      return;
    }

    try {
      await client.createAgent({
        name: newAgentName,
        description: newAgentDescription,
        model: newAgentModel,
        instructions: newAgentInstructions,
      });
      
      toast({
        title: 'Agent created',
        description: `${newAgentName} has been created successfully`,
      });
      
      // Reset form and close dialog
      setNewAgentName('');
      setNewAgentDescription('');
      setNewAgentModel('');
      setNewAgentInstructions('');
      setIsCreateDialogOpen(false);
      
      // Refresh agent list
      fetchAgents();
    } catch (error) {
      console.error('Error creating agent:', error);
      toast({
        title: 'Error creating agent',
        description: 'Could not create the agent',
        variant: 'destructive',
      });
    }
  };

  useEffect(() => {
    fetchAgents();
    fetchModels();
  }, []);

  return (
    <div className="container mx-auto py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Lumos Agents</h1>
        
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <PlusIcon className="mr-2 h-4 w-4" />
              Create Agent
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[425px]">
            <DialogHeader>
              <DialogTitle>Create New Agent</DialogTitle>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="name">Name</Label>
                <Input 
                  id="name" 
                  value={newAgentName} 
                  onChange={(e) => setNewAgentName(e.target.value)}
                  placeholder="My Agent" 
                />
              </div>
              
              <div className="grid gap-2">
                <Label htmlFor="description">Description</Label>
                <Input 
                  id="description" 
                  value={newAgentDescription} 
                  onChange={(e) => setNewAgentDescription(e.target.value)}
                  placeholder="A helpful assistant" 
                />
              </div>
              
              <div className="grid gap-2">
                <Label htmlFor="model">Model</Label>
                <Select value={newAgentModel} onValueChange={setNewAgentModel}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a model" />
                  </SelectTrigger>
                  <SelectContent>
                    {modelOptions.map((model) => (
                      <SelectItem key={model} value={model}>{model}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              
              <div className="grid gap-2">
                <Label htmlFor="instructions">Instructions</Label>
                <Textarea 
                  id="instructions" 
                  value={newAgentInstructions} 
                  onChange={(e) => setNewAgentInstructions(e.target.value)}
                  placeholder="You are a helpful assistant..."
                  rows={4}
                />
              </div>
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={handleCreateAgent}>
                Create
              </Button>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      <AgentsTable agents={agents} isLoading={isLoading} onRefresh={fetchAgents} />
    </div>
  );
} 