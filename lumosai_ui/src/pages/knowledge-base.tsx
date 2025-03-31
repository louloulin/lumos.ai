import { useState, useEffect } from 'react';
import { LomusaiClient } from '@lomusai/client-js';
import { useToast } from '@/components/ui/use-toast';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Upload, Database, Search, FileText, Trash2, RefreshCw, Plus } from 'lucide-react';
import { Progress } from '@/components/ui/progress';

export default function KnowledgeBasePage() {
  const [documents, setDocuments] = useState<any[]>([]);
  const [indexes, setIndexes] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [selectedIndex, setSelectedIndex] = useState('');
  const [isSearching, setIsSearching] = useState(false);
  const { toast } = useToast();

  // Document upload state
  const [isUploadDialogOpen, setIsUploadDialogOpen] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadingFile, setUploadingFile] = useState<File | null>(null);
  const [documentName, setDocumentName] = useState('');
  const [documentDescription, setDocumentDescription] = useState('');

  // Index creation state
  const [isCreateIndexDialogOpen, setIsCreateIndexDialogOpen] = useState(false);
  const [newIndexName, setNewIndexName] = useState('');
  const [newIndexDescription, setNewIndexDescription] = useState('');
  const [indexType, setIndexType] = useState('text');
  const [selectedDocuments, setSelectedDocuments] = useState<string[]>([]);
  const [isCreatingIndex, setIsCreatingIndex] = useState(false);

  const client = new LomusaiClient({
    baseUrl: import.meta.env.VITE_API_BASE_URL || '',
  });

  const fetchDocuments = async () => {
    try {
      const response = await client.getDocuments();
      setDocuments(response);
    } catch (error) {
      console.error('Error fetching documents:', error);
      toast({
        title: 'Error fetching documents',
        description: 'Could not retrieve documents',
        variant: 'destructive',
      });
    }
  };

  const fetchIndexes = async () => {
    try {
      const response = await client.getIndexes();
      setIndexes(response);
      if (response.length > 0 && !selectedIndex) {
        setSelectedIndex(response[0].id);
      }
    } catch (error) {
      console.error('Error fetching indexes:', error);
      toast({
        title: 'Error fetching indexes',
        description: 'Could not retrieve indexes',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      setUploadingFile(file);
      if (!documentName) {
        setDocumentName(file.name.split('.')[0]);
      }
    }
  };

  const handleDocumentUpload = async () => {
    if (!uploadingFile) {
      toast({
        title: 'No file selected',
        description: 'Please select a file to upload',
        variant: 'destructive',
      });
      return;
    }

    setIsUploading(true);
    setUploadProgress(0);

    try {
      // Setup simulated progress updates
      const progressInterval = setInterval(() => {
        setUploadProgress((prev) => {
          const newProgress = prev + 10;
          if (newProgress >= 90) {
            clearInterval(progressInterval);
            return 90;
          }
          return newProgress;
        });
      }, 500);

      // Upload the document
      const formData = new FormData();
      formData.append('file', uploadingFile);
      formData.append('name', documentName);
      formData.append('description', documentDescription);

      await client.uploadDocument(formData);

      clearInterval(progressInterval);
      setUploadProgress(100);

      toast({
        title: 'Document uploaded',
        description: 'Document has been uploaded successfully',
      });

      setTimeout(() => {
        setIsUploading(false);
        setUploadingFile(null);
        setDocumentName('');
        setDocumentDescription('');
        setUploadProgress(0);
        setIsUploadDialogOpen(false);
        fetchDocuments();
      }, 1000);
    } catch (error) {
      console.error('Error uploading document:', error);
      toast({
        title: 'Error uploading document',
        description: 'Could not upload the document',
        variant: 'destructive',
      });
      setIsUploading(false);
      setUploadProgress(0);
    }
  };

  const handleDeleteDocument = async (documentId: string) => {
    try {
      await client.deleteDocument(documentId);
      toast({
        title: 'Document deleted',
        description: 'Document has been deleted successfully',
      });
      fetchDocuments();
    } catch (error) {
      console.error('Error deleting document:', error);
      toast({
        title: 'Error deleting document',
        description: 'Could not delete the document',
        variant: 'destructive',
      });
    }
  };

  const handleCreateIndex = async () => {
    if (!newIndexName.trim()) {
      toast({
        title: 'Validation error',
        description: 'Index name is required',
        variant: 'destructive',
      });
      return;
    }

    if (selectedDocuments.length === 0) {
      toast({
        title: 'Validation error',
        description: 'Please select at least one document to index',
        variant: 'destructive',
      });
      return;
    }

    setIsCreatingIndex(true);

    try {
      await client.createIndex({
        name: newIndexName,
        description: newIndexDescription,
        type: indexType,
        documentIds: selectedDocuments,
      });

      toast({
        title: 'Index created',
        description: `${newIndexName} has been created successfully`,
      });

      setNewIndexName('');
      setNewIndexDescription('');
      setIndexType('text');
      setSelectedDocuments([]);
      setIsCreateIndexDialogOpen(false);
      fetchIndexes();
    } catch (error) {
      console.error('Error creating index:', error);
      toast({
        title: 'Error creating index',
        description: 'Could not create the index',
        variant: 'destructive',
      });
    } finally {
      setIsCreatingIndex(false);
    }
  };

  const handleDeleteIndex = async (indexId: string) => {
    try {
      await client.deleteIndex(indexId);
      toast({
        title: 'Index deleted',
        description: 'Index has been deleted successfully',
      });
      fetchIndexes();
    } catch (error) {
      console.error('Error deleting index:', error);
      toast({
        title: 'Error deleting index',
        description: 'Could not delete the index',
        variant: 'destructive',
      });
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim() || !selectedIndex) {
      return;
    }

    setIsSearching(true);
    setSearchResults([]);

    try {
      const results = await client.searchIndex(selectedIndex, searchQuery);
      setSearchResults(results);
    } catch (error) {
      console.error('Error searching index:', error);
      toast({
        title: 'Error searching',
        description: 'Could not perform search',
        variant: 'destructive',
      });
    } finally {
      setIsSearching(false);
    }
  };

  useEffect(() => {
    fetchDocuments();
    fetchIndexes();
  }, []);

  return (
    <div className="container mx-auto py-8">
      <h1 className="text-2xl font-bold mb-6">Knowledge Base</h1>

      <Tabs defaultValue="documents">
        <TabsList className="mb-4">
          <TabsTrigger value="documents">Documents</TabsTrigger>
          <TabsTrigger value="indexes">Indexes</TabsTrigger>
          <TabsTrigger value="search">Search</TabsTrigger>
        </TabsList>

        {/* Documents Tab */}
        <TabsContent value="documents">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-xl font-semibold">Documents</h2>
            <Dialog open={isUploadDialogOpen} onOpenChange={setIsUploadDialogOpen}>
              <DialogTrigger asChild>
                <Button>
                  <Upload className="mr-2 h-4 w-4" />
                  Upload Document
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Upload Document</DialogTitle>
                  <DialogDescription>
                    Upload a document to the knowledge base. Supported formats: PDF, TXT, DOCX, MD.
                  </DialogDescription>
                </DialogHeader>

                <div className="grid gap-4 py-4">
                  <div className="grid gap-2">
                    <Label htmlFor="document-file">File</Label>
                    <Input
                      id="document-file"
                      type="file"
                      accept=".pdf,.txt,.docx,.md"
                      onChange={handleFileChange}
                      disabled={isUploading}
                    />
                  </div>

                  <div className="grid gap-2">
                    <Label htmlFor="document-name">Name</Label>
                    <Input
                      id="document-name"
                      value={documentName}
                      onChange={(e) => setDocumentName(e.target.value)}
                      placeholder="Document name"
                      disabled={isUploading}
                    />
                  </div>

                  <div className="grid gap-2">
                    <Label htmlFor="document-description">Description (Optional)</Label>
                    <Textarea
                      id="document-description"
                      value={documentDescription}
                      onChange={(e) => setDocumentDescription(e.target.value)}
                      placeholder="Document description"
                      disabled={isUploading}
                    />
                  </div>

                  {isUploading && (
                    <div className="space-y-2">
                      <Progress value={uploadProgress} className="w-full" />
                      <p className="text-xs text-center">{uploadProgress}% uploaded</p>
                    </div>
                  )}
                </div>

                <DialogFooter>
                  <Button variant="outline" onClick={() => setIsUploadDialogOpen(false)} disabled={isUploading}>
                    Cancel
                  </Button>
                  <Button onClick={handleDocumentUpload} disabled={isUploading || !uploadingFile}>
                    {isUploading ? 'Uploading...' : 'Upload'}
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </div>

          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Size</TableHead>
                  <TableHead>Uploaded</TableHead>
                  <TableHead className="w-[100px]">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoading ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center py-8">
                      Loading documents...
                    </TableCell>
                  </TableRow>
                ) : documents.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center py-8">
                      No documents found. Upload one to get started.
                    </TableCell>
                  </TableRow>
                ) : (
                  documents.map((document) => (
                    <TableRow key={document.id}>
                      <TableCell className="font-medium">{document.name}</TableCell>
                      <TableCell className="max-w-[300px] truncate">{document.description || 'No description'}</TableCell>
                      <TableCell>{document.type}</TableCell>
                      <TableCell>{formatFileSize(document.size)}</TableCell>
                      <TableCell>{new Date(document.createdAt).toLocaleString()}</TableCell>
                      <TableCell>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDeleteDocument(document.id)}
                          className="text-destructive"
                        >
                          <Trash2 className="h-4 w-4" />
                          <span className="sr-only">Delete</span>
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </TabsContent>

        {/* Indexes Tab */}
        <TabsContent value="indexes">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-xl font-semibold">Indexes</h2>
            <Dialog open={isCreateIndexDialogOpen} onOpenChange={setIsCreateIndexDialogOpen}>
              <DialogTrigger asChild>
                <Button>
                  <Plus className="mr-2 h-4 w-4" />
                  Create Index
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Create Index</DialogTitle>
                  <DialogDescription>
                    Create a new index from existing documents.
                  </DialogDescription>
                </DialogHeader>

                <div className="grid gap-4 py-4">
                  <div className="grid gap-2">
                    <Label htmlFor="index-name">Name</Label>
                    <Input
                      id="index-name"
                      value={newIndexName}
                      onChange={(e) => setNewIndexName(e.target.value)}
                      placeholder="Index name"
                    />
                  </div>

                  <div className="grid gap-2">
                    <Label htmlFor="index-description">Description (Optional)</Label>
                    <Textarea
                      id="index-description"
                      value={newIndexDescription}
                      onChange={(e) => setNewIndexDescription(e.target.value)}
                      placeholder="Index description"
                    />
                  </div>

                  <div className="grid gap-2">
                    <Label htmlFor="index-type">Index Type</Label>
                    <Select value={indexType} onValueChange={setIndexType}>
                      <SelectTrigger>
                        <SelectValue placeholder="Select index type" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="text">Text</SelectItem>
                        <SelectItem value="vector">Vector</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="grid gap-2">
                    <Label>Select Documents</Label>
                    <div className="border rounded-md p-4 space-y-2 max-h-40 overflow-y-auto">
                      {documents.length === 0 ? (
                        <p className="text-sm text-muted-foreground">No documents available. Upload some first.</p>
                      ) : (
                        documents.map((document) => (
                          <div key={document.id} className="flex items-center space-x-2">
                            <Input
                              type="checkbox"
                              id={`doc-${document.id}`}
                              className="w-4 h-4"
                              checked={selectedDocuments.includes(document.id)}
                              onChange={(e) => {
                                if (e.target.checked) {
                                  setSelectedDocuments([...selectedDocuments, document.id]);
                                } else {
                                  setSelectedDocuments(selectedDocuments.filter(id => id !== document.id));
                                }
                              }}
                            />
                            <Label htmlFor={`doc-${document.id}`} className="text-sm">
                              {document.name}
                            </Label>
                          </div>
                        ))
                      )}
                    </div>
                  </div>
                </div>

                <DialogFooter>
                  <Button variant="outline" onClick={() => setIsCreateIndexDialogOpen(false)} disabled={isCreatingIndex}>
                    Cancel
                  </Button>
                  <Button onClick={handleCreateIndex} disabled={isCreatingIndex || documents.length === 0}>
                    {isCreatingIndex ? 'Creating...' : 'Create Index'}
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </div>

          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Documents</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="w-[100px]">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoading ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center py-8">
                      Loading indexes...
                    </TableCell>
                  </TableRow>
                ) : indexes.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-center py-8">
                      No indexes found. Create one to get started.
                    </TableCell>
                  </TableRow>
                ) : (
                  indexes.map((index) => (
                    <TableRow key={index.id}>
                      <TableCell className="font-medium">{index.name}</TableCell>
                      <TableCell className="max-w-[300px] truncate">{index.description || 'No description'}</TableCell>
                      <TableCell>{index.type}</TableCell>
                      <TableCell>{index.documentIds?.length || 0} documents</TableCell>
                      <TableCell>{new Date(index.createdAt).toLocaleString()}</TableCell>
                      <TableCell>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDeleteIndex(index.id)}
                          className="text-destructive"
                        >
                          <Trash2 className="h-4 w-4" />
                          <span className="sr-only">Delete</span>
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </TabsContent>

        {/* Search Tab */}
        <TabsContent value="search">
          <Card>
            <CardHeader>
              <CardTitle>Search Knowledge Base</CardTitle>
              <CardDescription>
                Search through your indexed documents for relevant information.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="grid gap-2">
                  <Label htmlFor="index-select">Select Index</Label>
                  <Select value={selectedIndex} onValueChange={setSelectedIndex} disabled={indexes.length === 0}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select an index" />
                    </SelectTrigger>
                    <SelectContent>
                      {indexes.map((index) => (
                        <SelectItem key={index.id} value={index.id}>
                          {index.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="search-query">Search Query</Label>
                  <div className="flex space-x-2">
                    <Input
                      id="search-query"
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      placeholder="Enter your search query"
                      disabled={isSearching || !selectedIndex}
                      className="flex-1"
                    />
                    <Button onClick={handleSearch} disabled={isSearching || !searchQuery.trim() || !selectedIndex}>
                      <Search className="h-4 w-4 mr-2" />
                      {isSearching ? 'Searching...' : 'Search'}
                    </Button>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {searchResults.length > 0 && (
            <Card className="mt-6">
              <CardHeader>
                <CardTitle>Search Results</CardTitle>
                <CardDescription>Found {searchResults.length} results for "{searchQuery}"</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {searchResults.map((result, index) => (
                    <div key={index} className="border rounded-md p-4">
                      <div className="flex items-center space-x-2 mb-2">
                        <FileText className="h-4 w-4" />
                        <h3 className="font-medium">{result.document?.name || 'Document'}</h3>
                        {result.score && (
                          <span className="text-xs bg-muted px-2 py-1 rounded-full">
                            Score: {Math.round(result.score * 100)}%
                          </span>
                        )}
                      </div>
                      <p className="text-sm whitespace-pre-line">{result.content}</p>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}

// Utility function to format file size
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
} 