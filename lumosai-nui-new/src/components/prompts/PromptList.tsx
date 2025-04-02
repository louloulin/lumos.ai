import React, { useState, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Badge } from '../ui/badge';
import { Switch } from '../ui/switch';
import { 
  Plus, Search, Tag, Clock, User, Bookmark, Filter, ArrowUpDown, 
  FileText, Copy, ExternalLink, Share2, MoreVertical, Sparkles
} from 'lucide-react';
import { Popover, PopoverContent, PopoverTrigger } from '../ui/popover';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '../ui/dropdown-menu';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { promptsService, PromptTemplate } from '../../services/prompts';

const EmptyState: React.FC<{ onCreate: () => void }> = ({ onCreate }) => (
  <div className="flex flex-col items-center justify-center p-10 text-center">
    <div className="w-20 h-20 bg-emerald-600/20 rounded-full flex items-center justify-center mb-4">
      <FileText size={32} className="text-emerald-400" />
    </div>
    <h3 className="text-xl font-semibold mb-2">没有提示模板</h3>
    <p className="text-gray-400 mb-6 max-w-md">
      创建您的第一个提示模板，以便在AI工作流和应用中重复使用。
    </p>
    <Button onClick={onCreate} className="bg-emerald-600 hover:bg-emerald-700">
      <Plus size={16} className="mr-2" />
      创建提示模板
    </Button>
  </div>
);

const PromptCard: React.FC<{ 
  template: PromptTemplate; 
  onCopy: (template: PromptTemplate) => void;
}> = ({ template, onCopy }) => {
  const navigate = useNavigate();
  
  // 提取变量数量
  const variablesCount = template.variables?.length || 0;
  
  return (
    <Card className="bg-gray-900/70 hover:bg-gray-900 border-gray-700 transition-colors duration-150 overflow-hidden">
      <CardHeader className="p-4 pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg font-medium cursor-pointer" onClick={() => navigate(`/prompts/${template.id}`)}>
            {template.name}
          </CardTitle>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8 text-gray-400">
                <MoreVertical size={16} />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent className="bg-gray-800 border-gray-700">
              <DropdownMenuItem 
                className="cursor-pointer hover:bg-gray-700"
                onClick={() => navigate(`/prompts/${template.id}`)}
              >
                <FileText size={14} className="mr-2" />
                编辑模板
              </DropdownMenuItem>
              <DropdownMenuItem 
                className="cursor-pointer hover:bg-gray-700"
                onClick={() => onCopy(template)}
              >
                <Copy size={14} className="mr-2" />
                复制模板
              </DropdownMenuItem>
              <DropdownMenuSeparator className="bg-gray-700" />
              <DropdownMenuItem 
                className="cursor-pointer hover:bg-gray-700"
                onClick={() => navigate(`/prompts/${template.id}?tab=test`)}
              >
                <Sparkles size={14} className="mr-2" />
                测试运行
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
        <CardDescription className="line-clamp-2 mt-1 text-gray-400">
          {template.description}
        </CardDescription>
      </CardHeader>
      <CardContent className="p-4 pt-2">
        <div className="h-20 overflow-hidden relative mt-1 mb-2">
          <pre className="text-xs text-gray-400 whitespace-pre-wrap font-mono bg-black/20 p-2 rounded border border-gray-800">
            {template.content.length > 150 
              ? template.content.substring(0, 150) + '...' 
              : template.content}
          </pre>
          <div className="absolute inset-x-0 bottom-0 h-12 bg-gradient-to-t from-gray-900/90 to-transparent"></div>
        </div>
        
        <div className="flex flex-wrap gap-1.5 mt-3">
          {template.tags?.map(tag => (
            <Badge key={tag} className="bg-blue-900/40 text-blue-300 text-xs">
              {tag}
            </Badge>
          ))}
          {template.category && (
            <Badge className="bg-purple-900/40 text-purple-300 text-xs">
              {template.category}
            </Badge>
          )}
          {template.model && (
            <Badge className="bg-amber-900/40 text-amber-300 text-xs">
              {template.model}
            </Badge>
          )}
        </div>
      </CardContent>
      <CardFooter className="px-4 py-3 bg-black/20 border-t border-gray-800 flex items-center justify-between text-xs text-gray-400">
        <div className="flex items-center space-x-3">
          {variablesCount > 0 && (
            <div className="flex items-center">
              <Tag size={12} className="mr-1 text-emerald-400" />
              <span>{variablesCount} 个变量</span>
            </div>
          )}
          <div className="flex items-center">
            <Clock size={12} className="mr-1" />
            <span>{new Date(template.updatedAt).toLocaleDateString()}</span>
          </div>
        </div>
        <div className="flex items-center">
          {template.isPublic ? (
            <Badge className="bg-emerald-900/40 text-emerald-300 text-xs">公开</Badge>
          ) : (
            <Badge className="bg-gray-800 text-gray-400 text-xs">私有</Badge>
          )}
        </div>
      </CardFooter>
    </Card>
  );
};

const PromptList: React.FC = () => {
  const navigate = useNavigate();
  const [templates, setTemplates] = useState<PromptTemplate[]>([]);
  const [filteredTemplates, setFilteredTemplates] = useState<PromptTemplate[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  
  // 过滤选项
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [showPublicOnly, setShowPublicOnly] = useState(false);
  const [sortBy, setSortBy] = useState<'updated' | 'name' | 'created'>('updated');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  
  // 所有可用的标签、类别和模型
  const [availableTags, setAvailableTags] = useState<string[]>([]);
  const [availableCategories, setAvailableCategories] = useState<string[]>([]);
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  
  // 加载提示模板
  useEffect(() => {
    const loadTemplates = async () => {
      setIsLoading(true);
      try {
        // 假设当前用户ID
        const userId = 'user-1';
        const templates = await promptsService.getPromptTemplates(userId);
        setTemplates(templates);
        
        // 提取所有可用的标签、类别和模型
        const tags = new Set<string>();
        const categories = new Set<string>();
        const models = new Set<string>();
        
        templates.forEach(template => {
          template.tags?.forEach(tag => tags.add(tag));
          if (template.category) categories.add(template.category);
          if (template.model) models.add(template.model);
        });
        
        setAvailableTags(Array.from(tags));
        setAvailableCategories(Array.from(categories));
        setAvailableModels(Array.from(models));
      } catch (error) {
        console.error('Failed to load templates:', error);
      } finally {
        setIsLoading(false);
      }
    };

    loadTemplates();
  }, []);
  
  // 根据搜索和过滤条件更新过滤后的模板
  useEffect(() => {
    let filtered = [...templates];
    
    // 应用搜索
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(template => 
        template.name.toLowerCase().includes(query) ||
        template.description.toLowerCase().includes(query) ||
        template.content.toLowerCase().includes(query) ||
        template.tags.some(tag => tag.toLowerCase().includes(query))
      );
    }
    
    // 应用类别过滤器
    if (selectedCategory) {
      filtered = filtered.filter(template => template.category === selectedCategory);
    }
    
    // 应用模型过滤器
    if (selectedModel) {
      filtered = filtered.filter(template => template.model === selectedModel);
    }
    
    // 应用标签过滤器
    if (selectedTags.length > 0) {
      filtered = filtered.filter(template => 
        selectedTags.some(tag => template.tags.includes(tag))
      );
    }
    
    // 应用公开/私有过滤器
    if (showPublicOnly) {
      filtered = filtered.filter(template => template.isPublic);
    }
    
    // 应用排序
    filtered.sort((a, b) => {
      let comparison = 0;
      
      if (sortBy === 'updated') {
        comparison = new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime();
      } else if (sortBy === 'created') {
        comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
      } else if (sortBy === 'name') {
        comparison = a.name.localeCompare(b.name);
      }
      
      return sortOrder === 'desc' ? -comparison : comparison;
    });
    
    setFilteredTemplates(filtered);
  }, [templates, searchQuery, selectedCategory, selectedModel, selectedTags, showPublicOnly, sortBy, sortOrder]);
  
  // 复制模板
  const handleCopyTemplate = async (template: PromptTemplate) => {
    try {
      const { id, createdAt, updatedAt, createdBy, ...templateData } = template;
      
      const newTemplate = await promptsService.createPromptTemplate({
        ...templateData,
        name: `${templateData.name} (副本)`,
      }, 'user-1'); // 假设当前用户ID
      
      // 刷新模板列表
      setTemplates(prev => [...prev, newTemplate]);
      
      // 导航到新模板
      navigate(`/prompts/${newTemplate.id}`);
    } catch (error) {
      console.error('Failed to copy template:', error);
    }
  };
  
  // 清除所有过滤器
  const clearFilters = () => {
    setSearchQuery('');
    setSelectedCategory('');
    setSelectedModel('');
    setSelectedTags([]);
    setShowPublicOnly(false);
    setSortBy('updated');
    setSortOrder('desc');
  };
  
  // 检查是否应用了任何过滤器
  const hasActiveFilters = Boolean(
    searchQuery || selectedCategory || selectedModel || 
    selectedTags.length > 0 || showPublicOnly || 
    sortBy !== 'updated' || sortOrder !== 'desc'
  );
  
  // 处理标签切换
  const toggleTag = (tag: string) => {
    setSelectedTags(prev => 
      prev.includes(tag) 
        ? prev.filter(t => t !== tag) 
        : [...prev, tag]
    );
  };

  return (
    <div className="container mx-auto p-6 max-w-7xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold mb-1">提示模板库</h1>
          <p className="text-gray-400">创建和管理可复用的提示模板</p>
        </div>
        
        <Button 
          onClick={() => navigate('/prompts/new')}
          className="bg-emerald-600 hover:bg-emerald-700 text-white"
        >
          <Plus size={16} className="mr-2" />
          新建模板
        </Button>
      </div>
      
      <div className="mb-6">
        <div className="flex flex-col md:flex-row gap-4">
          <div className="relative flex-1">
            <Search size={18} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="搜索提示模板..."
              className="pl-10 bg-gray-800 border-gray-700 focus:ring-emerald-500/30 focus:border-emerald-500/60"
            />
          </div>
          
          <div className="flex flex-wrap items-center gap-3">
            <Popover>
              <PopoverTrigger asChild>
                <Button 
                  variant="outline"
                  className={`bg-gray-800 border-gray-700 ${selectedCategory ? 'text-emerald-400 border-emerald-500/50' : ''}`}
                >
                  <Filter size={16} className="mr-2" />
                  {selectedCategory || '类别'}
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-64 p-3 bg-gray-800 border-gray-700">
                <div className="space-y-2">
                  <h4 className="font-medium text-sm">按类别筛选</h4>
                  <Select
                    value={selectedCategory}
                    onValueChange={setSelectedCategory}
                  >
                    <SelectTrigger className="w-full bg-gray-900 border-gray-700">
                      <SelectValue placeholder="选择类别" />
                    </SelectTrigger>
                    <SelectContent className="bg-gray-900 border-gray-700">
                      <SelectItem value="">全部类别</SelectItem>
                      {availableCategories.map(category => (
                        <SelectItem key={category} value={category}>{category}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </PopoverContent>
            </Popover>
            
            <Popover>
              <PopoverTrigger asChild>
                <Button 
                  variant="outline"
                  className={`bg-gray-800 border-gray-700 ${selectedModel ? 'text-emerald-400 border-emerald-500/50' : ''}`}
                >
                  <Sparkles size={16} className="mr-2" />
                  {selectedModel || '模型'}
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-64 p-3 bg-gray-800 border-gray-700">
                <div className="space-y-2">
                  <h4 className="font-medium text-sm">按模型筛选</h4>
                  <Select
                    value={selectedModel}
                    onValueChange={setSelectedModel}
                  >
                    <SelectTrigger className="w-full bg-gray-900 border-gray-700">
                      <SelectValue placeholder="选择模型" />
                    </SelectTrigger>
                    <SelectContent className="bg-gray-900 border-gray-700">
                      <SelectItem value="">全部模型</SelectItem>
                      {availableModels.map(model => (
                        <SelectItem key={model} value={model}>{model}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </PopoverContent>
            </Popover>
            
            <Popover>
              <PopoverTrigger asChild>
                <Button 
                  variant="outline"
                  className={`bg-gray-800 border-gray-700 ${selectedTags.length > 0 ? 'text-emerald-400 border-emerald-500/50' : ''}`}
                >
                  <Tag size={16} className="mr-2" />
                  {selectedTags.length 
                    ? `${selectedTags.length} 个标签` 
                    : '标签'}
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-64 p-3 bg-gray-800 border-gray-700">
                <div className="space-y-3">
                  <h4 className="font-medium text-sm">按标签筛选</h4>
                  {availableTags.length === 0 ? (
                    <p className="text-gray-400 text-sm">没有可用的标签</p>
                  ) : (
                    <div className="flex flex-wrap gap-2">
                      {availableTags.map(tag => (
                        <Badge 
                          key={tag} 
                          className={`cursor-pointer ${
                            selectedTags.includes(tag)
                              ? 'bg-emerald-600/40 text-emerald-300'
                              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                          }`}
                          onClick={() => toggleTag(tag)}
                        >
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
              </PopoverContent>
            </Popover>
            
            <Button
              variant="outline"
              className={`bg-gray-800 border-gray-700 ${showPublicOnly ? 'text-emerald-400 border-emerald-500/50' : ''}`}
              onClick={() => setShowPublicOnly(!showPublicOnly)}
            >
              <Share2 size={16} className="mr-2" />
              {showPublicOnly ? '仅公开' : '全部'}
            </Button>
            
            <Popover>
              <PopoverTrigger asChild>
                <Button 
                  variant="outline"
                  className="bg-gray-800 border-gray-700"
                >
                  <ArrowUpDown size={16} className="mr-2" />
                  排序
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-64 p-3 bg-gray-800 border-gray-700">
                <div className="space-y-3">
                  <h4 className="font-medium text-sm">排序</h4>
                  <div className="space-y-2">
                    <Select
                      value={sortBy}
                      onValueChange={(value: any) => setSortBy(value)}
                    >
                      <SelectTrigger className="w-full bg-gray-900 border-gray-700">
                        <SelectValue placeholder="排序依据" />
                      </SelectTrigger>
                      <SelectContent className="bg-gray-900 border-gray-700">
                        <SelectItem value="updated">更新时间</SelectItem>
                        <SelectItem value="created">创建时间</SelectItem>
                        <SelectItem value="name">名称</SelectItem>
                      </SelectContent>
                    </Select>
                    
                    <Select
                      value={sortOrder}
                      onValueChange={(value: any) => setSortOrder(value)}
                    >
                      <SelectTrigger className="w-full bg-gray-900 border-gray-700">
                        <SelectValue placeholder="排序方向" />
                      </SelectTrigger>
                      <SelectContent className="bg-gray-900 border-gray-700">
                        <SelectItem value="desc">降序</SelectItem>
                        <SelectItem value="asc">升序</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              </PopoverContent>
            </Popover>
            
            {hasActiveFilters && (
              <Button 
                variant="ghost"
                onClick={clearFilters}
                className="text-gray-400 hover:text-white"
              >
                清除
              </Button>
            )}
          </div>
        </div>
      </div>
      
      {isLoading ? (
        <div className="flex items-center justify-center h-64">
          <div className="flex flex-col items-center space-y-4">
            <div className="animate-spin h-8 w-8 border-4 border-emerald-500 border-opacity-20 border-t-emerald-500 rounded-full" />
            <p className="text-gray-400">加载提示模板...</p>
          </div>
        </div>
      ) : filteredTemplates.length === 0 ? (
        templates.length === 0 ? (
          <EmptyState onCreate={() => navigate('/prompts/new')} />
        ) : (
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <div className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center mb-4">
              <Search size={24} className="text-gray-400" />
            </div>
            <h3 className="text-xl font-semibold mb-2">未找到匹配的提示模板</h3>
            <p className="text-gray-400 mb-6 max-w-md">
              尝试调整您的搜索条件或过滤器，以查看更多结果。
            </p>
            <Button onClick={clearFilters} className="bg-gray-700 hover:bg-gray-600">
              清除所有过滤器
            </Button>
          </div>
        )
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredTemplates.map(template => (
            <PromptCard 
              key={template.id} 
              template={template} 
              onCopy={handleCopyTemplate}
            />
          ))}
        </div>
      )}
    </div>
  );
};

export default PromptList; 