import React, { useEffect, useState } from 'react';
import { Input } from '../ui/input';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { 
  LucideBrain, 
  LucideWrench, 
  LucideUserCog, 
  LucideGitBranch, 
  LucideSearch, 
  LucideTag,
  LucidePlus,
  LucideMoreHorizontal
} from 'lucide-react';
import { 
  ResourceType, 
  Model, 
  Tool, 
  Agent, 
  Workflow,
  getModels,
  getTools,
  getAgents,
  getWorkflows,
  searchResources
} from '../../services/resources';

// 资源卡片 - 通用属性
interface ResourceCardProps {
  id: string;
  name: string;
  description: string;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  onClick?: () => void;
}

// 模型卡片组件
const ModelCard: React.FC<ResourceCardProps & { provider: string; contextWindow: number; }> = ({
  name, description, provider, contextWindow, tags, updatedAt, onClick
}) => (
  <Card className="p-4 hover:bg-secondary/50 cursor-pointer" onClick={onClick}>
    <div className="flex items-start justify-between">
      <div className="flex gap-3">
        <div className="p-2 bg-primary/10 rounded-full">
          <LucideBrain className="text-primary" size={20} />
        </div>
        <div>
          <h3 className="font-medium">{name}</h3>
          <p className="text-sm text-muted-foreground">{description}</p>
          <div className="flex gap-2 mt-2">
            <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded-full">{provider}</span>
            <span className="text-xs bg-gray-100 text-gray-800 px-2 py-1 rounded-full">{contextWindow.toLocaleString()} tokens</span>
          </div>
        </div>
      </div>
      <Button variant="ghost" size="icon" onClick={(e) => { e.stopPropagation(); }}>
        <LucideMoreHorizontal size={16} />
      </Button>
    </div>
    <div className="mt-3 flex flex-wrap gap-1">
      {tags.map(tag => (
        <span key={tag} className="text-xs bg-primary/10 text-primary-foreground px-2 py-0.5 rounded-full">
          #{tag}
        </span>
      ))}
    </div>
    <div className="mt-2 text-xs text-muted-foreground">
      Updated: {new Date(updatedAt).toLocaleDateString()}
    </div>
  </Card>
);

// 工具卡片组件
const ToolCard: React.FC<ResourceCardProps & { category: string; }> = ({
  name, description, category, tags, updatedAt, onClick
}) => (
  <Card className="p-4 hover:bg-secondary/50 cursor-pointer" onClick={onClick}>
    <div className="flex items-start justify-between">
      <div className="flex gap-3">
        <div className="p-2 bg-primary/10 rounded-full">
          <LucideWrench className="text-primary" size={20} />
        </div>
        <div>
          <h3 className="font-medium">{name}</h3>
          <p className="text-sm text-muted-foreground">{description}</p>
          <div className="flex gap-2 mt-2">
            <span className="text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full">{category}</span>
          </div>
        </div>
      </div>
      <Button variant="ghost" size="icon" onClick={(e) => { e.stopPropagation(); }}>
        <LucideMoreHorizontal size={16} />
      </Button>
    </div>
    <div className="mt-3 flex flex-wrap gap-1">
      {tags.map(tag => (
        <span key={tag} className="text-xs bg-primary/10 text-primary-foreground px-2 py-0.5 rounded-full">
          #{tag}
        </span>
      ))}
    </div>
    <div className="mt-2 text-xs text-muted-foreground">
      Updated: {new Date(updatedAt).toLocaleDateString()}
    </div>
  </Card>
);

// 代理卡片组件
const AgentCard: React.FC<ResourceCardProps & { status: string; model: string; }> = ({
  name, description, status, model, tags, updatedAt, onClick
}) => (
  <Card className="p-4 hover:bg-secondary/50 cursor-pointer" onClick={onClick}>
    <div className="flex items-start justify-between">
      <div className="flex gap-3">
        <div className="p-2 bg-primary/10 rounded-full">
          <LucideUserCog className="text-primary" size={20} />
        </div>
        <div>
          <h3 className="font-medium">{name}</h3>
          <p className="text-sm text-muted-foreground">{description}</p>
          <div className="flex gap-2 mt-2">
            <span className={`text-xs px-2 py-1 rounded-full ${
              status === 'active' 
                ? 'bg-green-100 text-green-800' 
                : status === 'draft' 
                ? 'bg-yellow-100 text-yellow-800' 
                : 'bg-gray-100 text-gray-800'
            }`}>
              {status}
            </span>
            <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded-full">{model}</span>
          </div>
        </div>
      </div>
      <Button variant="ghost" size="icon" onClick={(e) => { e.stopPropagation(); }}>
        <LucideMoreHorizontal size={16} />
      </Button>
    </div>
    <div className="mt-3 flex flex-wrap gap-1">
      {tags.map(tag => (
        <span key={tag} className="text-xs bg-primary/10 text-primary-foreground px-2 py-0.5 rounded-full">
          #{tag}
        </span>
      ))}
    </div>
    <div className="mt-2 text-xs text-muted-foreground">
      Updated: {new Date(updatedAt).toLocaleDateString()}
    </div>
  </Card>
);

// 工作流卡片组件
const WorkflowCard: React.FC<ResourceCardProps & { status: string; nodeCount: number; }> = ({
  name, description, status, nodeCount, tags, updatedAt, onClick
}) => (
  <Card className="p-4 hover:bg-secondary/50 cursor-pointer" onClick={onClick}>
    <div className="flex items-start justify-between">
      <div className="flex gap-3">
        <div className="p-2 bg-primary/10 rounded-full">
          <LucideGitBranch className="text-primary" size={20} />
        </div>
        <div>
          <h3 className="font-medium">{name}</h3>
          <p className="text-sm text-muted-foreground">{description}</p>
          <div className="flex gap-2 mt-2">
            <span className={`text-xs px-2 py-1 rounded-full ${
              status === 'active' 
                ? 'bg-green-100 text-green-800' 
                : status === 'draft' 
                ? 'bg-yellow-100 text-yellow-800' 
                : 'bg-gray-100 text-gray-800'
            }`}>
              {status}
            </span>
            <span className="text-xs bg-purple-100 text-purple-800 px-2 py-1 rounded-full">
              {nodeCount} nodes
            </span>
          </div>
        </div>
      </div>
      <Button variant="ghost" size="icon" onClick={(e) => { e.stopPropagation(); }}>
        <LucideMoreHorizontal size={16} />
      </Button>
    </div>
    <div className="mt-3 flex flex-wrap gap-1">
      {tags.map(tag => (
        <span key={tag} className="text-xs bg-primary/10 text-primary-foreground px-2 py-0.5 rounded-full">
          #{tag}
        </span>
      ))}
    </div>
    <div className="mt-2 text-xs text-muted-foreground">
      Updated: {new Date(updatedAt).toLocaleDateString()}
    </div>
  </Card>
);

// 空状态组件
const EmptyState: React.FC<{ type: ResourceType; onCreate?: () => void }> = ({ type, onCreate }) => {
  const typeMap = {
    model: { label: 'Models', icon: <LucideBrain size={40} /> },
    tool: { label: 'Tools', icon: <LucideWrench size={40} /> },
    agent: { label: 'Agents', icon: <LucideUserCog size={40} /> },
    workflow: { label: 'Workflows', icon: <LucideGitBranch size={40} /> }
  };

  return (
    <div className="flex flex-col items-center justify-center p-8 text-center">
      <div className="p-4 bg-primary/10 rounded-full mb-4">
        {typeMap[type].icon}
      </div>
      <h3 className="text-xl font-medium mb-2">No {typeMap[type].label} Found</h3>
      <p className="text-muted-foreground mb-6">
        Get started by creating your first {type}
      </p>
      <Button onClick={onCreate}>
        <LucidePlus className="mr-2" size={16} />
        Create {type.charAt(0).toUpperCase() + type.slice(1)}
      </Button>
    </div>
  );
};

// 加载状态组件
const LoadingState: React.FC = () => (
  <div className="flex justify-center items-center h-[40vh]">
    <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-primary"></div>
  </div>
);

// 搜索栏组件
const ResourceSearch: React.FC<{ 
  onSearch: (query: string) => void;
  onCreateNew?: () => void;
}> = ({ onSearch, onCreateNew }) => {
  const [query, setQuery] = useState('');
  
  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    onSearch(query);
  };
  
  return (
    <div className="flex items-center space-x-2">
      <form onSubmit={handleSearch} className="flex-1 flex">
        <div className="relative flex-1">
          <LucideSearch className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground" size={16} />
          <Input
            className="pl-9"
            placeholder="Search resources..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        <Button type="submit" className="ml-2" variant="secondary">Search</Button>
      </form>
      {onCreateNew && (
        <Button onClick={onCreateNew}>
          <LucidePlus size={16} className="mr-2" />
          New
        </Button>
      )}
    </div>
  );
};

// 资源过滤器组件
const ResourceFilters: React.FC<{
  tags: string[];
  selectedTags: string[];
  onTagSelect: (tag: string) => void;
}> = ({ tags, selectedTags, onTagSelect }) => (
  <div className="mb-4">
    <div className="flex items-center mb-2">
      <LucideTag size={16} className="mr-2" />
      <span className="text-sm font-medium">Filter by tags:</span>
    </div>
    <div className="flex flex-wrap gap-2">
      {tags.map(tag => (
        <span
          key={tag}
          className={`text-xs px-2 py-1 rounded-full cursor-pointer ${
            selectedTags.includes(tag)
              ? 'bg-primary text-primary-foreground'
              : 'bg-primary/10 text-primary hover:bg-primary/20'
          }`}
          onClick={() => onTagSelect(tag)}
        >
          #{tag}
        </span>
      ))}
    </div>
  </div>
);

// 主资源浏览器组件
export const ResourceBrowser: React.FC = () => {
  const [activeTab, setActiveTab] = useState<ResourceType>('model');
  const [loading, setLoading] = useState(true);
  const [models, setModels] = useState<Model[]>([]);
  const [tools, setTools] = useState<Tool[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [allTags, setAllTags] = useState<string[]>([]);
  const [selectedTags, setSelectedTags] = useState<string[]>([]);

  useEffect(() => {
    const loadResources = async () => {
      setLoading(true);
      try {
        // 获取当前标签页的资源
        switch (activeTab) {
          case 'model':
            const fetchedModels = await getModels();
            setModels(fetchedModels);
            
            // 提取所有标签
            const modelTags = fetchedModels.flatMap(model => model.tags);
            setAllTags([...new Set(modelTags)]);
            break;
            
          case 'tool':
            const fetchedTools = await getTools();
            setTools(fetchedTools);
            
            // 提取所有标签
            const toolTags = fetchedTools.flatMap(tool => tool.tags);
            setAllTags([...new Set(toolTags)]);
            break;
            
          case 'agent':
            const fetchedAgents = await getAgents();
            setAgents(fetchedAgents);
            
            // 提取所有标签
            const agentTags = fetchedAgents.flatMap(agent => agent.tags);
            setAllTags([...new Set(agentTags)]);
            break;
            
          case 'workflow':
            const fetchedWorkflows = await getWorkflows();
            setWorkflows(fetchedWorkflows);
            
            // 提取所有标签
            const workflowTags = fetchedWorkflows.flatMap(workflow => workflow.tags);
            setAllTags([...new Set(workflowTags)]);
            break;
        }
      } catch (error) {
        console.error(`Failed to load ${activeTab}s:`, error);
      } finally {
        setLoading(false);
      }
    };

    // 切换标签时重置搜索和过滤器状态
    setSearchQuery('');
    setSelectedTags([]);
    
    loadResources();
  }, [activeTab]);

  // 搜索资源
  const handleSearch = async (query: string) => {
    setLoading(true);
    setSearchQuery(query);
    
    try {
      // 使用搜索API
      const types = activeTab ? [activeTab] : undefined;
      const tags = selectedTags.length > 0 ? selectedTags : undefined;
      
      const results = await searchResources(query, types, tags);
      
      // 根据资源类型更新不同的状态
      const modelResults = results.filter(r => r.type === 'model') as Model[];
      const toolResults = results.filter(r => r.type === 'tool') as Tool[];
      const agentResults = results.filter(r => r.type === 'agent') as Agent[];
      const workflowResults = results.filter(r => r.type === 'workflow') as Workflow[];
      
      if (activeTab === 'model' || !activeTab) setModels(modelResults);
      if (activeTab === 'tool' || !activeTab) setTools(toolResults);
      if (activeTab === 'agent' || !activeTab) setAgents(agentResults);
      if (activeTab === 'workflow' || !activeTab) setWorkflows(workflowResults);
      
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setLoading(false);
    }
  };

  // 处理标签选择
  const handleTagSelect = (tag: string) => {
    setSelectedTags(prevTags => {
      // 如果标签已经被选中，则取消选择
      if (prevTags.includes(tag)) {
        return prevTags.filter(t => t !== tag);
      }
      // 否则，添加到选中标签中
      return [...prevTags, tag];
    });
    
    // 如果有搜索查询或标签选择，重新搜索
    if (searchQuery || selectedTags.length > 0) {
      handleSearch(searchQuery);
    }
  };

  // 创建新资源的处理函数（模拟）
  const handleCreateResource = (type: ResourceType) => {
    console.log(`Create new ${type}`);
    // 实际应用中，这里将导航到创建表单或打开模态框
  };

  // 资源选择处理函数（模拟）
  const handleResourceSelect = (id: string, type: ResourceType) => {
    console.log(`Selected ${type} with id ${id}`);
    // 实际应用中，这里将导航到资源详情页或打开模态框
  };

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Resource Browser</h1>
      </div>
      
      <ResourceSearch 
        onSearch={handleSearch}
        onCreateNew={() => handleCreateResource(activeTab)}
      />
      
      {selectedTags.length > 0 && (
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Filtered by:</span>
          {selectedTags.map(tag => (
            <span 
              key={tag}
              className="bg-primary text-primary-foreground text-xs px-2 py-1 rounded-full flex items-center"
            >
              #{tag}
              <button
                className="ml-1 hover:bg-primary-foreground/20 rounded-full p-0.5"
                onClick={() => handleTagSelect(tag)}
              >
                ✕
              </button>
            </span>
          ))}
          <Button 
            variant="ghost" 
            size="sm" 
            onClick={() => setSelectedTags([])}
          >
            Clear all
          </Button>
        </div>
      )}
      
      <Tabs 
        defaultValue="model" 
        value={activeTab} 
        onValueChange={(value) => setActiveTab(value as ResourceType)}
        className="space-y-4"
      >
        <TabsList>
          <TabsTrigger value="model" className="flex items-center">
            <LucideBrain className="mr-2" size={16} />
            Models
          </TabsTrigger>
          <TabsTrigger value="tool" className="flex items-center">
            <LucideWrench className="mr-2" size={16} />
            Tools
          </TabsTrigger>
          <TabsTrigger value="agent" className="flex items-center">
            <LucideUserCog className="mr-2" size={16} />
            Agents
          </TabsTrigger>
          <TabsTrigger value="workflow" className="flex items-center">
            <LucideGitBranch className="mr-2" size={16} />
            Workflows
          </TabsTrigger>
        </TabsList>
        
        {/* 显示过滤器，仅当有标签时 */}
        {allTags.length > 0 && (
          <ResourceFilters 
            tags={allTags}
            selectedTags={selectedTags}
            onTagSelect={handleTagSelect}
          />
        )}
        
        {/* 模型标签页 */}
        <TabsContent value="model" className="space-y-4">
          {loading ? (
            <LoadingState />
          ) : models.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {models.map(model => (
                <ModelCard
                  key={model.id}
                  id={model.id}
                  name={model.name}
                  description={model.description}
                  provider={model.provider}
                  contextWindow={model.contextWindow}
                  tags={model.tags}
                  createdAt={model.createdAt}
                  updatedAt={model.updatedAt}
                  onClick={() => handleResourceSelect(model.id, 'model')}
                />
              ))}
            </div>
          ) : (
            <EmptyState 
              type="model"
              onCreate={() => handleCreateResource('model')}
            />
          )}
        </TabsContent>
        
        {/* 工具标签页 */}
        <TabsContent value="tool" className="space-y-4">
          {loading ? (
            <LoadingState />
          ) : tools.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {tools.map(tool => (
                <ToolCard
                  key={tool.id}
                  id={tool.id}
                  name={tool.name}
                  description={tool.description}
                  category={tool.category}
                  tags={tool.tags}
                  createdAt={tool.createdAt}
                  updatedAt={tool.updatedAt}
                  onClick={() => handleResourceSelect(tool.id, 'tool')}
                />
              ))}
            </div>
          ) : (
            <EmptyState 
              type="tool"
              onCreate={() => handleCreateResource('tool')}
            />
          )}
        </TabsContent>
        
        {/* 代理标签页 */}
        <TabsContent value="agent" className="space-y-4">
          {loading ? (
            <LoadingState />
          ) : agents.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {agents.map(agent => (
                <AgentCard
                  key={agent.id}
                  id={agent.id}
                  name={agent.name}
                  description={agent.description}
                  status={agent.status}
                  model={agent.model}
                  tags={agent.tags}
                  createdAt={agent.createdAt}
                  updatedAt={agent.updatedAt}
                  onClick={() => handleResourceSelect(agent.id, 'agent')}
                />
              ))}
            </div>
          ) : (
            <EmptyState 
              type="agent"
              onCreate={() => handleCreateResource('agent')}
            />
          )}
        </TabsContent>
        
        {/* 工作流标签页 */}
        <TabsContent value="workflow" className="space-y-4">
          {loading ? (
            <LoadingState />
          ) : workflows.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {workflows.map(workflow => (
                <WorkflowCard
                  key={workflow.id}
                  id={workflow.id}
                  name={workflow.name}
                  description={workflow.description}
                  status={workflow.status}
                  nodeCount={workflow.nodes.length}
                  tags={workflow.tags}
                  createdAt={workflow.createdAt}
                  updatedAt={workflow.updatedAt}
                  onClick={() => handleResourceSelect(workflow.id, 'workflow')}
                />
              ))}
            </div>
          ) : (
            <EmptyState 
              type="workflow"
              onCreate={() => handleCreateResource('workflow')}
            />
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}; 