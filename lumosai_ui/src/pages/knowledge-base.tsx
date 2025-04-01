import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { 
  Search, 
  Upload, 
  FileText, 
  Folder, 
  Trash2, 
  Plus, 
  RefreshCw,
  Clock,
  FileQuestion 
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { 
  Dialog, 
  DialogContent, 
  DialogDescription, 
  DialogFooter, 
  DialogHeader, 
  DialogTitle,
  DialogTrigger 
} from '@/components/ui/dialog';
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from '@/components/ui/table';
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '@/components/ui/select';
import { Label } from '@/components/ui/label';
import { useToast } from '@/components/ui/use-toast';
import { Checkbox } from '@/components/ui/checkbox';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Separator } from '@/components/ui/separator';
import { Progress } from '@/components/ui/progress';

// 文档接口
interface Document {
  id: string;
  name: string;
  type: string;
  size: number;
  uploadedAt: string;
  collection: string;
  status: 'processing' | 'indexed' | 'failed';
  metadata?: {
    pageCount?: number;
    author?: string;
    createdAt?: string;
    lastModified?: string;
  };
}

// 集合接口
interface Collection {
  id: string;
  name: string;
  description: string;
  documentCount: number;
  createdAt: string;
}

// 示例文档数据
const exampleDocuments: Document[] = [
  {
    id: 'doc-1',
    name: 'LumosAI白皮书.pdf',
    type: 'pdf',
    size: 2540000,
    uploadedAt: '2023-06-15T08:30:00Z',
    collection: 'company-docs',
    status: 'indexed',
    metadata: {
      pageCount: 42,
      author: '技术团队',
      createdAt: '2023-05-10T10:00:00Z',
      lastModified: '2023-06-01T14:20:00Z',
    },
  },
  {
    id: 'doc-2',
    name: '产品说明书.docx',
    type: 'docx',
    size: 1240000,
    uploadedAt: '2023-06-10T09:15:00Z',
    collection: 'product-docs',
    status: 'indexed',
    metadata: {
      pageCount: 15,
      author: '产品团队',
      createdAt: '2023-05-05T11:30:00Z',
      lastModified: '2023-06-08T16:45:00Z',
    },
  },
  {
    id: 'doc-3',
    name: '用户手册.pdf',
    type: 'pdf',
    size: 3600000,
    uploadedAt: '2023-06-05T14:20:00Z',
    collection: 'user-guides',
    status: 'indexed',
    metadata: {
      pageCount: 65,
      author: '文档团队',
      createdAt: '2023-04-20T09:00:00Z',
      lastModified: '2023-05-30T11:10:00Z',
    },
  },
  {
    id: 'doc-4',
    name: '技术架构.pptx',
    type: 'pptx',
    size: 5100000,
    uploadedAt: '2023-06-01T10:45:00Z',
    collection: 'tech-docs',
    status: 'indexed',
    metadata: {
      pageCount: 28,
      author: '架构师',
      createdAt: '2023-05-15T08:30:00Z',
      lastModified: '2023-05-28T17:00:00Z',
    },
  },
  {
    id: 'doc-5',
    name: '季度报告.xlsx',
    type: 'xlsx',
    size: 980000,
    uploadedAt: '2023-05-25T16:30:00Z',
    collection: 'reports',
    status: 'indexed',
    metadata: {
      author: '财务部',
      createdAt: '2023-05-20T14:00:00Z',
      lastModified: '2023-05-24T11:30:00Z',
    },
  },
];

// 示例集合数据
const exampleCollections: Collection[] = [
  {
    id: 'company-docs',
    name: '公司文档',
    description: '包含公司介绍、白皮书等文档',
    documentCount: 8,
    createdAt: '2023-04-01T10:00:00Z',
  },
  {
    id: 'product-docs',
    name: '产品文档',
    description: '产品相关的说明书和技术文档',
    documentCount: 15,
    createdAt: '2023-04-05T14:30:00Z',
  },
  {
    id: 'user-guides',
    name: '用户指南',
    description: '面向用户的使用手册和教程',
    documentCount: 12,
    createdAt: '2023-04-10T09:15:00Z',
  },
  {
    id: 'tech-docs',
    name: '技术文档',
    description: '技术架构和开发规范文档',
    documentCount: 20,
    createdAt: '2023-04-15T11:45:00Z',
  },
  {
    id: 'reports',
    name: '报告集',
    description: '各类数据报告和分析文档',
    documentCount: 6,
    createdAt: '2023-04-20T16:00:00Z',
  },
];

export default function KnowledgeBasePage() {
  const navigate = useNavigate();
  const { toast } = useToast();
  const [documents, setDocuments] = useState<Document[]>([]);
  const [collections, setCollections] = useState<Collection[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isUploadDialogOpen, setIsUploadDialogOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCollection, setSelectedCollection] = useState<string | null>(null);
  const [selectedFileType, setSelectedFileType] = useState<string | null>(null);
  const [selectedDocuments, setSelectedDocuments] = useState<string[]>([]);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [isUploading, setIsUploading] = useState(false);
  const [newCollection, setNewCollection] = useState({
    name: '',
    description: ''
  });
  const [isNewCollectionDialogOpen, setIsNewCollectionDialogOpen] = useState(false);
  
  useEffect(() => {
    // 模拟加载数据
    const loadData = async () => {
      setIsLoading(true);
      try {
        // 在实际应用中，这里会是API调用
        await new Promise(resolve => setTimeout(resolve, 800)); // 模拟网络延迟
        
        setDocuments(exampleDocuments);
        setCollections(exampleCollections);
      } catch (error) {
        toast({
          title: "加载失败",
          description: "无法加载知识库数据",
          variant: "destructive",
        });
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, [toast]);

  // 过滤文档的函数
  const filteredDocuments = documents.filter(doc => {
    const matchesSearch = searchQuery === '' || 
      doc.name.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesCollection = selectedCollection === null || 
      doc.collection === selectedCollection;
    
    const matchesType = selectedFileType === null || 
      doc.type === selectedFileType;
    
    return matchesSearch && matchesCollection && matchesType;
  });

  // 计算统计信息
  const stats = {
    totalDocuments: documents.length,
    totalCollections: collections.length,
    totalSize: documents.reduce((acc, doc) => acc + doc.size, 0),
    documentTypes: [...new Set(documents.map(doc => doc.type))].length
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const handleFileUpload = (files: FileList | null) => {
    if (!files || files.length === 0) return;
    
    setIsUploading(true);
    setUploadProgress(0);
    
    // 模拟上传进度
    const interval = setInterval(() => {
      setUploadProgress(prev => {
        if (prev >= 100) {
          clearInterval(interval);
          return 100;
        }
        return prev + 10;
      });
    }, 300);
    
    // 模拟上传完成
    setTimeout(() => {
      clearInterval(interval);
      setUploadProgress(100);
      
      // 创建新文档对象
      const newDocs: Document[] = Array.from(files).map((file, index) => {
        const fileExtension = file.name.split('.').pop() || '';
        return {
          id: `doc-new-${Date.now()}-${index}`,
          name: file.name,
          type: fileExtension,
          size: file.size,
          uploadedAt: new Date().toISOString(),
          collection: selectedCollection || 'default',
          status: 'processing',
        };
      });
      
      setDocuments(prev => [...newDocs, ...prev]);
      
      // 模拟处理完成
      setTimeout(() => {
        setDocuments(prev => 
          prev.map(doc => 
            newDocs.some(newDoc => newDoc.id === doc.id) 
              ? { ...doc, status: 'indexed' } 
              : doc
          )
        );
        
        setIsUploading(false);
        setIsUploadDialogOpen(false);
        
        toast({
          title: "上传成功",
          description: `已成功上传 ${files.length} 个文档`,
        });
      }, 1500);
    }, 3000);
  };

  const handleDeleteDocuments = () => {
    if (selectedDocuments.length === 0) return;
    
    // 从列表中移除选中的文档
    setDocuments(prev => 
      prev.filter(doc => !selectedDocuments.includes(doc.id))
    );
    
    toast({
      title: "删除成功",
      description: `已删除 ${selectedDocuments.length} 个文档`,
    });
    
    // 清空选择
    setSelectedDocuments([]);
  };

  const handleCreateCollection = () => {
    if (!newCollection.name.trim()) {
      toast({
        title: "创建失败",
        description: "集合名称不能为空",
        variant: "destructive",
      });
      return;
    }
    
    const newCollectionObj: Collection = {
      id: `collection-${Date.now()}`,
      name: newCollection.name,
      description: newCollection.description,
      documentCount: 0,
      createdAt: new Date().toISOString(),
    };
    
    setCollections(prev => [newCollectionObj, ...prev]);
    setNewCollection({ name: '', description: '' });
    setIsNewCollectionDialogOpen(false);
    
    toast({
      title: "创建成功",
      description: `已创建新集合"${newCollection.name}"`,
    });
  };

  const toggleDocumentSelection = (docId: string) => {
    setSelectedDocuments(prev => 
      prev.includes(docId)
        ? prev.filter(id => id !== docId)
        : [...prev, docId]
    );
  };

  const selectAllDocuments = () => {
    if (selectedDocuments.length === filteredDocuments.length) {
      // 如果已经全选，则取消全选
      setSelectedDocuments([]);
    } else {
      // 否则全选
      setSelectedDocuments(filteredDocuments.map(doc => doc.id));
    }
  };

  return (
    <div className="container py-8">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center mb-6 gap-4">
        <div>
          <h1 className="text-3xl font-bold">知识库管理</h1>
          <p className="text-muted-foreground">上传、搜索和管理您的文档集</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => setIsNewCollectionDialogOpen(true)}>
            <Folder className="mr-2 h-4 w-4" />
            新建集合
          </Button>
          <Button onClick={() => setIsUploadDialogOpen(true)}>
            <Upload className="mr-2 h-4 w-4" />
            上传文档
          </Button>
        </div>
      </div>
      
      {/* 统计卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">总文档数</p>
                <p className="text-2xl font-bold">{stats.totalDocuments}</p>
              </div>
              <FileText className="h-8 w-8 text-muted-foreground" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">总集合数</p>
                <p className="text-2xl font-bold">{stats.totalCollections}</p>
              </div>
              <Folder className="h-8 w-8 text-muted-foreground" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">存储容量</p>
                <p className="text-2xl font-bold">{formatFileSize(stats.totalSize)}</p>
              </div>
              <FileQuestion className="h-8 w-8 text-muted-foreground" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">文档类型</p>
                <p className="text-2xl font-bold">{stats.documentTypes}</p>
              </div>
              <FileText className="h-8 w-8 text-muted-foreground" />
            </div>
          </CardContent>
        </Card>
      </div>
      
      <Tabs defaultValue="documents" className="mb-6">
        <TabsList>
          <TabsTrigger value="documents">文档</TabsTrigger>
          <TabsTrigger value="collections">集合</TabsTrigger>
        </TabsList>
        
        <TabsContent value="documents">
          <Card>
            <CardHeader className="p-4">
              <div className="flex flex-col md:flex-row justify-between space-y-2 md:space-y-0 md:items-center">
                <div className="relative w-full md:w-96">
                  <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="搜索文档..."
                    className="pl-8"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                  />
                </div>
                
                <div className="flex items-center space-x-2">
                  <Select value={selectedCollection || ''} onValueChange={(val) => setSelectedCollection(val || null)}>
                    <SelectTrigger className="w-[180px]">
                      <SelectValue placeholder="所有集合" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="">所有集合</SelectItem>
                      {collections.map((collection) => (
                        <SelectItem key={collection.id} value={collection.id}>
                          {collection.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  
                  <Select value={selectedFileType || ''} onValueChange={(val) => setSelectedFileType(val || null)}>
                    <SelectTrigger className="w-[150px]">
                      <SelectValue placeholder="所有类型" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="">所有类型</SelectItem>
                      {[...new Set(documents.map(doc => doc.type))].map((type) => (
                        <SelectItem key={type} value={type}>
                          {type.toUpperCase()}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  
                  {selectedDocuments.length > 0 && (
                    <Button 
                      variant="destructive" 
                      size="sm" 
                      onClick={handleDeleteDocuments}
                    >
                      <Trash2 className="h-4 w-4 mr-1" />
                      删除 ({selectedDocuments.length})
                    </Button>
                  )}
                </div>
              </div>
            </CardHeader>
            
            <CardContent className="p-0">
              {isLoading ? (
                <div className="p-8 flex justify-center">
                  <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : filteredDocuments.length === 0 ? (
                <div className="p-8 text-center">
                  <FileQuestion className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
                  <h3 className="text-lg font-medium">没有找到文档</h3>
                  <p className="text-muted-foreground mb-4">尝试修改搜索条件或上传新文档</p>
                  <Button onClick={() => setIsUploadDialogOpen(true)}>
                    <Upload className="mr-2 h-4 w-4" />
                    上传文档
                  </Button>
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[40px]">
                        <Checkbox 
                          checked={selectedDocuments.length > 0 && selectedDocuments.length === filteredDocuments.length}
                          onCheckedChange={selectAllDocuments}
                        />
                      </TableHead>
                      <TableHead>文档名称</TableHead>
                      <TableHead>类型</TableHead>
                      <TableHead>大小</TableHead>
                      <TableHead>状态</TableHead>
                      <TableHead>集合</TableHead>
                      <TableHead>上传时间</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {filteredDocuments.map((doc) => (
                      <TableRow key={doc.id}>
                        <TableCell>
                          <Checkbox 
                            checked={selectedDocuments.includes(doc.id)}
                            onCheckedChange={() => toggleDocumentSelection(doc.id)}
                          />
                        </TableCell>
                        <TableCell className="font-medium">{doc.name}</TableCell>
                        <TableCell>
                          <Badge variant="outline">{doc.type.toUpperCase()}</Badge>
                        </TableCell>
                        <TableCell>{formatFileSize(doc.size)}</TableCell>
                        <TableCell>
                          {doc.status === 'indexed' ? (
                            <Badge variant="default" className="bg-green-600">已索引</Badge>
                          ) : doc.status === 'processing' ? (
                            <Badge variant="secondary" className="bg-yellow-500">处理中</Badge>
                          ) : (
                            <Badge variant="destructive">失败</Badge>
                          )}
                        </TableCell>
                        <TableCell>
                          {collections.find(c => c.id === doc.collection)?.name || doc.collection}
                        </TableCell>
                        <TableCell className="text-muted-foreground">
                          {new Date(doc.uploadedAt).toLocaleString('zh-CN', {
                            year: 'numeric',
                            month: '2-digit',
                            day: '2-digit',
                            hour: '2-digit',
                            minute: '2-digit'
                          })}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
            
            <CardFooter className="py-2 px-4 border-t">
              <div className="text-sm text-muted-foreground">
                显示 {filteredDocuments.length} 个文档，共 {documents.length} 个
              </div>
            </CardFooter>
          </Card>
        </TabsContent>
        
        <TabsContent value="collections">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {collections.map((collection) => (
              <Card key={collection.id}>
                <CardHeader>
                  <CardTitle>{collection.name}</CardTitle>
                  <CardDescription>{collection.description}</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">文档数量:</span>
                    <span className="font-medium">{collection.documentCount}</span>
                  </div>
                  <div className="flex justify-between text-sm mt-1">
                    <span className="text-muted-foreground">创建时间:</span>
                    <span className="font-medium">
                      {new Date(collection.createdAt).toLocaleDateString('zh-CN')}
                    </span>
                  </div>
                </CardContent>
                <CardFooter className="flex justify-between">
                  <Button 
                    variant="ghost" 
                    size="sm"
                    onClick={() => setSelectedCollection(collection.id)}
                  >
                    查看文档
                  </Button>
                  <Button 
                    variant="outline" 
                    size="icon"
                    onClick={() => {
                      // 实际应用中，这里应该有一个确认对话框
                      setCollections(prev => prev.filter(c => c.id !== collection.id));
                      toast({
                        title: "删除成功",
                        description: `已删除集合"${collection.name}"`,
                      });
                    }}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </CardFooter>
              </Card>
            ))}
            
            {/* 添加新集合卡片 */}
            <Card className="border-dashed cursor-pointer" onClick={() => setIsNewCollectionDialogOpen(true)}>
              <CardContent className="flex flex-col items-center justify-center h-full py-8">
                <Plus className="h-10 w-10 text-muted-foreground mb-4" />
                <p className="text-muted-foreground">添加新集合</p>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
      
      {/* 上传对话框 */}
      <Dialog open={isUploadDialogOpen} onOpenChange={setIsUploadDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>上传文档</DialogTitle>
            <DialogDescription>
              上传文档到知识库，支持PDF、Word、Excel、PowerPoint等多种格式
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label>选择集合</Label>
              <Select 
                value={selectedCollection || ''}
                onValueChange={(val) => setSelectedCollection(val || null)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="选择集合" />
                </SelectTrigger>
                <SelectContent>
                  {collections.map((collection) => (
                    <SelectItem key={collection.id} value={collection.id}>
                      {collection.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            
            {!isUploading ? (
              <div 
                className="border-2 border-dashed rounded-md p-8 text-center hover:bg-muted/50 transition-colors cursor-pointer"
                onClick={() => {
                  const input = document.createElement('input');
                  input.type = 'file';
                  input.multiple = true;
                  input.accept = '.pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.txt,.md,.json';
                  input.onchange = (e) => handleFileUpload((e.target as HTMLInputElement).files);
                  input.click();
                }}
              >
                <Upload className="h-10 w-10 text-muted-foreground mx-auto mb-4" />
                <p className="text-muted-foreground mb-1">拖拽文件到此处或点击上传</p>
                <p className="text-xs text-muted-foreground">
                  支持 PDF, Word, Excel, PowerPoint, TXT, Markdown 等格式
                </p>
              </div>
            ) : (
              <div className="space-y-2">
                <div className="flex justify-between text-sm mb-1">
                  <span>上传进度</span>
                  <span>{uploadProgress}%</span>
                </div>
                <Progress value={uploadProgress} max={100} />
                <p className="text-xs text-center text-muted-foreground mt-2">
                  正在上传...请勿关闭窗口
                </p>
              </div>
            )}
          </div>
          
          <DialogFooter>
            <Button 
              variant="outline" 
              onClick={() => setIsUploadDialogOpen(false)}
              disabled={isUploading}
            >
              取消
            </Button>
            <Button 
              onClick={() => {
                const input = document.createElement('input');
                input.type = 'file';
                input.multiple = true;
                input.accept = '.pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.txt,.md,.json';
                input.onchange = (e) => handleFileUpload((e.target as HTMLInputElement).files);
                input.click();
              }}
              disabled={isUploading}
            >
              选择文件
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      
      {/* 新建集合对话框 */}
      <Dialog open={isNewCollectionDialogOpen} onOpenChange={setIsNewCollectionDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新建集合</DialogTitle>
            <DialogDescription>
              创建一个新的文档集合，用于组织和管理相关文档
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label htmlFor="collection-name">集合名称</Label>
              <Input 
                id="collection-name" 
                value={newCollection.name}
                onChange={(e) => setNewCollection(prev => ({ ...prev, name: e.target.value }))}
                placeholder="输入集合名称"
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="collection-description">描述（可选）</Label>
              <Input 
                id="collection-description" 
                value={newCollection.description}
                onChange={(e) => setNewCollection(prev => ({ ...prev, description: e.target.value }))}
                placeholder="简要描述这个集合的用途"
              />
            </div>
          </div>
          
          <DialogFooter>
            <Button 
              variant="outline" 
              onClick={() => setIsNewCollectionDialogOpen(false)}
            >
              取消
            </Button>
            <Button onClick={handleCreateCollection}>
              创建
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
} 