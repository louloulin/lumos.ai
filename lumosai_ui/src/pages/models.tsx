import { useState, useEffect } from 'react';
import { LomusaiClient } from '@lomusai/client-js';
import { useToast } from '@/components/ui/use-toast';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter } from '@/components/ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Slider } from '@/components/ui/slider';
import { Plus, Trash2, RefreshCw, Zap, CopyCheck, Star } from 'lucide-react';

// Helper component for displaying test responses
const ModelResponseCard = ({ response }: { response: any }) => {
  return (
    <Card className="mt-4">
      <CardHeader>
        <CardTitle className="text-sm flex items-center">
          <CopyCheck className="h-4 w-4 mr-2" />
          Model Response
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="whitespace-pre-line text-sm">{response.content}</div>
      </CardContent>
      <CardFooter className="border-t px-6 py-3">
        <div className="flex items-center justify-between w-full text-xs text-muted-foreground">
          <div>Model: {response.model}</div>
          <div>Tokens: {response.usage?.total_tokens || 'N/A'}</div>
          <div>Time: {response.latency ? `${(response.latency / 1000).toFixed(2)}s` : 'N/A'}</div>
        </div>
      </CardFooter>
    </Card>
  );
};

export default function ModelsPage() {
  const [models, setModels] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedModel, setSelectedModel] = useState<any>(null);
  const [isAddModelDialogOpen, setIsAddModelDialogOpen] = useState(false);
  const { toast } = useToast();

  // New model form state
  const [newModelName, setNewModelName] = useState('');
  const [newModelProvider, setNewModelProvider] = useState('openai');
  const [newModelEndpoint, setNewModelEndpoint] = useState('');
  const [newModelApiKey, setNewModelApiKey] = useState('');
  const [newModelIsDefault, setNewModelIsDefault] = useState(false);

  // Model testing state
  const [testPrompt, setTestPrompt] = useState('Explain what a large language model is in simple terms.');
  const [testResponse, setTestResponse] = useState<any>(null);
  const [isTesting, setIsTesting] = useState(false);

  // Model parameters
  const [temperature, setTemperature] = useState(0.7);
  const [maxTokens, setMaxTokens] = useState(1000);
  const [topP, setTopP] = useState(1);
  const [frequencyPenalty, setFrequencyPenalty] = useState(0);
  const [presencePenalty, setPresencePenalty] = useState(0);

  const client = new LomusaiClient({
    baseUrl: import.meta.env.VITE_API_BASE_URL || '',
  });

  const fetchModels = async () => {
    setIsLoading(true);
    try {
      const response = await client.getModels();
      setModels(response);
      
      // Select the default model if available
      const defaultModel = response.find((model: any) => model.isDefault);
      if (defaultModel) {
        setSelectedModel(defaultModel);
      } else if (response.length > 0) {
        setSelectedModel(response[0]);
      }
    } catch (error) {
      console.error('Error fetching models:', error);
      toast({
        title: 'Error fetching models',
        description: 'Could not retrieve the list of models',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchModels();
  }, []);

  const handleAddModel = async () => {
    if (!newModelName.trim() || !newModelProvider.trim()) {
      toast({
        title: 'Validation error',
        description: 'Model name and provider are required',
        variant: 'destructive',
      });
      return;
    }

    try {
      await client.addModel({
        name: newModelName,
        provider: newModelProvider,
        endpoint: newModelEndpoint,
        apiKey: newModelApiKey,
        isDefault: newModelIsDefault,
      });

      toast({
        title: 'Model added',
        description: `${newModelName} has been added successfully`,
      });

      // Reset form and close dialog
      setNewModelName('');
      setNewModelProvider('openai');
      setNewModelEndpoint('');
      setNewModelApiKey('');
      setNewModelIsDefault(false);
      setIsAddModelDialogOpen(false);

      // Refresh model list
      fetchModels();
    } catch (error) {
      console.error('Error adding model:', error);
      toast({
        title: 'Error adding model',
        description: 'Could not add the model',
        variant: 'destructive',
      });
    }
  };

  const handleDeleteModel = async (modelId: string) => {
    try {
      await client.deleteModel(modelId);
      toast({
        title: 'Model deleted',
        description: 'Model has been deleted successfully',
      });
      
      // If the deleted model was selected, clear selection
      if (selectedModel && selectedModel.id === modelId) {
        setSelectedModel(null);
      }
      
      fetchModels();
    } catch (error) {
      console.error('Error deleting model:', error);
      toast({
        title: 'Error deleting model',
        description: 'Could not delete the model',
        variant: 'destructive',
      });
    }
  };

  const handleSetDefaultModel = async (modelId: string) => {
    try {
      await client.setDefaultModel(modelId);
      toast({
        title: 'Default model updated',
        description: 'Default model has been updated successfully',
      });
      fetchModels();
    } catch (error) {
      console.error('Error setting default model:', error);
      toast({
        title: 'Error updating default model',
        description: 'Could not update the default model',
        variant: 'destructive',
      });
    }
  };

  const handleTestModel = async () => {
    if (!selectedModel || !testPrompt.trim()) {
      toast({
        title: 'Validation error',
        description: 'Select a model and enter a prompt',
        variant: 'destructive',
      });
      return;
    }

    setIsTesting(true);
    setTestResponse(null);

    try {
      const response = await client.testModel(selectedModel.id, {
        prompt: testPrompt,
        parameters: {
          temperature,
          max_tokens: maxTokens,
          top_p: topP,
          frequency_penalty: frequencyPenalty,
          presence_penalty: presencePenalty,
        },
      });

      setTestResponse(response);
    } catch (error) {
      console.error('Error testing model:', error);
      toast({
        title: 'Error testing model',
        description: 'Could not test the model',
        variant: 'destructive',
      });
    } finally {
      setIsTesting(false);
    }
  };

  return (
    <div className="container mx-auto py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">LLM Models</h1>

        <Dialog open={isAddModelDialogOpen} onOpenChange={setIsAddModelDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Add Model
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[500px]">
            <DialogHeader>
              <DialogTitle>Add New Model</DialogTitle>
            </DialogHeader>

            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="model-name">Model Name</Label>
                <Input
                  id="model-name"
                  value={newModelName}
                  onChange={(e) => setNewModelName(e.target.value)}
                  placeholder="e.g., gpt-4-turbo"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="model-provider">Provider</Label>
                <Select value={newModelProvider} onValueChange={setNewModelProvider}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a provider" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="openai">OpenAI</SelectItem>
                    <SelectItem value="anthropic">Anthropic</SelectItem>
                    <SelectItem value="google">Google</SelectItem>
                    <SelectItem value="mistral">Mistral</SelectItem>
                    <SelectItem value="ollama">Ollama (Local)</SelectItem>
                    <SelectItem value="custom">Custom</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="model-endpoint">
                  Endpoint URL {newModelProvider !== 'custom' && <span className="text-muted-foreground">(Optional)</span>}
                </Label>
                <Input
                  id="model-endpoint"
                  value={newModelEndpoint}
                  onChange={(e) => setNewModelEndpoint(e.target.value)}
                  placeholder="https://api.example.com/v1/completions"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="model-api-key">API Key</Label>
                <Input
                  id="model-api-key"
                  type="password"
                  value={newModelApiKey}
                  onChange={(e) => setNewModelApiKey(e.target.value)}
                  placeholder="Enter API key"
                />
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="model-default"
                  checked={newModelIsDefault}
                  onCheckedChange={setNewModelIsDefault}
                />
                <Label htmlFor="model-default">Set as default model</Label>
              </div>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setIsAddModelDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={handleAddModel}>
                Add Model
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid md:grid-cols-3 gap-6">
        {/* Models List */}
        <div className="md:col-span-1">
          <Card>
            <CardHeader>
              <CardTitle className="flex justify-between items-center">
                Available Models
                <Button variant="ghost" size="icon" onClick={fetchModels} disabled={isLoading}>
                  <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
                </Button>
              </CardTitle>
              <CardDescription>Select a model to configure and test</CardDescription>
            </CardHeader>
            <CardContent className="p-0">
              {isLoading ? (
                <div className="p-6 text-center">Loading models...</div>
              ) : models.length === 0 ? (
                <div className="p-6 text-center">No models configured. Add one to get started.</div>
              ) : (
                <div className="divide-y">
                  {models.map((model) => (
                    <div
                      key={model.id}
                      className={`p-4 cursor-pointer hover:bg-muted flex justify-between items-center ${
                        selectedModel?.id === model.id ? 'bg-muted' : ''
                      }`}
                      onClick={() => setSelectedModel(model)}
                    >
                      <div className="flex items-center space-x-3">
                        {model.isDefault && <Star className="h-4 w-4 text-amber-500" />}
                        <div>
                          <div className="font-medium">{model.name}</div>
                          <div className="text-xs text-muted-foreground">{model.provider}</div>
                        </div>
                      </div>
                      <div className="flex items-center space-x-1">
                        {!model.isDefault && (
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleSetDefaultModel(model.id);
                            }}
                            title="Set as default"
                          >
                            <Star className="h-3.5 w-3.5" />
                          </Button>
                        )}
                        <Button
                          variant="ghost"
                          size="icon"
                          className="text-destructive"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDeleteModel(model.id);
                          }}
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Model Configuration and Testing */}
        <div className="md:col-span-2">
          {selectedModel ? (
            <Tabs defaultValue="test">
              <TabsList className="mb-4">
                <TabsTrigger value="test">Test Model</TabsTrigger>
                <TabsTrigger value="parameters">Parameters</TabsTrigger>
                <TabsTrigger value="info">Model Info</TabsTrigger>
              </TabsList>

              {/* Test Model Tab */}
              <TabsContent value="test">
                <Card>
                  <CardHeader>
                    <CardTitle>Test {selectedModel.name}</CardTitle>
                    <CardDescription>Enter a prompt to test this model's response</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      <div className="grid gap-2">
                        <Label htmlFor="test-prompt">Prompt</Label>
                        <Textarea
                          id="test-prompt"
                          value={testPrompt}
                          onChange={(e) => setTestPrompt(e.target.value)}
                          placeholder="Enter your prompt here..."
                          rows={5}
                        />
                      </div>
                    </div>
                  </CardContent>
                  <CardFooter className="justify-between">
                    <div className="text-sm text-muted-foreground">
                      Model: <span className="font-medium">{selectedModel.name}</span>
                    </div>
                    <Button onClick={handleTestModel} disabled={isTesting}>
                      <Zap className="mr-2 h-4 w-4" />
                      {isTesting ? 'Processing...' : 'Run Test'}
                    </Button>
                  </CardFooter>
                </Card>

                {testResponse && <ModelResponseCard response={testResponse} />}
              </TabsContent>

              {/* Parameters Tab */}
              <TabsContent value="parameters">
                <Card>
                  <CardHeader>
                    <CardTitle>Model Parameters</CardTitle>
                    <CardDescription>Configure parameters for {selectedModel.name}</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-6">
                      <div className="space-y-2">
                        <div className="flex justify-between">
                          <Label htmlFor="temperature">Temperature: {temperature}</Label>
                          <span className="text-xs text-muted-foreground">Controls randomness (0-2)</span>
                        </div>
                        <Slider
                          id="temperature"
                          min={0}
                          max={2}
                          step={0.1}
                          value={[temperature]}
                          onValueChange={(value) => setTemperature(value[0])}
                        />
                      </div>

                      <div className="space-y-2">
                        <div className="flex justify-between">
                          <Label htmlFor="max-tokens">Max Tokens: {maxTokens}</Label>
                          <span className="text-xs text-muted-foreground">Maximum response length</span>
                        </div>
                        <Slider
                          id="max-tokens"
                          min={1}
                          max={4000}
                          step={1}
                          value={[maxTokens]}
                          onValueChange={(value) => setMaxTokens(value[0])}
                        />
                      </div>

                      <div className="space-y-2">
                        <div className="flex justify-between">
                          <Label htmlFor="top-p">Top P: {topP}</Label>
                          <span className="text-xs text-muted-foreground">Token selection strategy (0-1)</span>
                        </div>
                        <Slider
                          id="top-p"
                          min={0}
                          max={1}
                          step={0.05}
                          value={[topP]}
                          onValueChange={(value) => setTopP(value[0])}
                        />
                      </div>

                      <div className="space-y-2">
                        <div className="flex justify-between">
                          <Label htmlFor="frequency-penalty">Frequency Penalty: {frequencyPenalty}</Label>
                          <span className="text-xs text-muted-foreground">Repetition reduction (-2 to 2)</span>
                        </div>
                        <Slider
                          id="frequency-penalty"
                          min={-2}
                          max={2}
                          step={0.1}
                          value={[frequencyPenalty]}
                          onValueChange={(value) => setFrequencyPenalty(value[0])}
                        />
                      </div>

                      <div className="space-y-2">
                        <div className="flex justify-between">
                          <Label htmlFor="presence-penalty">Presence Penalty: {presencePenalty}</Label>
                          <span className="text-xs text-muted-foreground">Topic switching (-2 to 2)</span>
                        </div>
                        <Slider
                          id="presence-penalty"
                          min={-2}
                          max={2}
                          step={0.1}
                          value={[presencePenalty]}
                          onValueChange={(value) => setPresencePenalty(value[0])}
                        />
                      </div>
                    </div>
                  </CardContent>
                  <CardFooter>
                    <Button onClick={handleTestModel} disabled={isTesting}>
                      <Zap className="mr-2 h-4 w-4" />
                      Test with These Parameters
                    </Button>
                  </CardFooter>
                </Card>
              </TabsContent>

              {/* Model Info Tab */}
              <TabsContent value="info">
                <Card>
                  <CardHeader>
                    <CardTitle>{selectedModel.name}</CardTitle>
                    <CardDescription>Model information and configuration</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <Label className="text-muted-foreground">Provider</Label>
                          <div className="font-medium">{selectedModel.provider}</div>
                        </div>
                        <div>
                          <Label className="text-muted-foreground">Status</Label>
                          <div className="font-medium">{selectedModel.isDefault ? 'Default' : 'Active'}</div>
                        </div>
                        {selectedModel.endpoint && (
                          <div className="col-span-2">
                            <Label className="text-muted-foreground">Endpoint</Label>
                            <div className="font-medium truncate">{selectedModel.endpoint}</div>
                          </div>
                        )}
                        {selectedModel.contextWindow && (
                          <div>
                            <Label className="text-muted-foreground">Context Window</Label>
                            <div className="font-medium">{selectedModel.contextWindow.toLocaleString()} tokens</div>
                          </div>
                        )}
                        {selectedModel.tokenLimit && (
                          <div>
                            <Label className="text-muted-foreground">Token Limit</Label>
                            <div className="font-medium">{selectedModel.tokenLimit.toLocaleString()} tokens</div>
                          </div>
                        )}
                        {selectedModel.added && (
                          <div>
                            <Label className="text-muted-foreground">Added</Label>
                            <div className="font-medium">{new Date(selectedModel.added).toLocaleString()}</div>
                          </div>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          ) : (
            <Card>
              <CardContent className="p-8 text-center">
                <p className="text-muted-foreground">
                  {models.length === 0
                    ? 'No models configured. Add a model to get started.'
                    : 'Select a model from the list to configure and test it.'}
                </p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
} 