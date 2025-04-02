import React, { useEffect, useState } from 'react';
import { Input } from '../ui/input';
import { Button } from '../ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { 
  Brain, 
  Wrench, 
  UserCog, 
  GitBranch, 
  Search, 
  Tag,
  Plus,
  MoreHorizontal,
  FolderOpen,
  Filter,
  SlidersHorizontal,
  ExternalLink,
  ChevronDown
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
  <div className="p-3 border-b border-[#2E2E2E] hover:bg-[#2E2E2E] cursor-pointer group transition-colors" onClick={onClick}>
    <div className="flex items-start justify-between">
      <div className="flex gap-3">
        <div className="p-1.5 rounded-md bg-[#2E2E2E] group-hover:bg-[#3E3E3E] text-emerald-500">
          <Brain size={16} />
        </div>
        <div>
          <h3 className="font-medium text-white text-sm">{name}</h3>
          <p className="text-xs text-gray-400 mt-1 line-clamp-1">{description}</p>
          <div className="flex gap-1.5 mt-2">
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#2C3494]/20 text-indigo-400 border border-[#2C3494]/20">
              {provider}
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#2E2E2E] group-hover:bg-[#3E3E3E] text-gray-400 border border-[#2E2E2E] group-hover:border-[#3E3E3E]">
              {contextWindow.toLocaleString()} tokens
            </span>
          </div>
        </div>
      </div>
      <Button variant="ghost" size="icon" className="h-7 w-7 rounded-md opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-white hover:bg-[#3E3E3E]" onClick={(e) => { e.stopPropagation(); }}>
        <MoreHorizontal size={14} />
      </Button>
    </div>
    {tags.length > 0 && (
      <div className="mt-3 flex flex-wrap gap-1 pl-8">
        {tags.slice(0, 3).map(tag => (
          <span key={tag} className="text-[10px] text-gray-400 px-1.5 py-0.5 rounded border border-[#2E2E2E] group-hover:border-[#3E3E3E] bg-[#1C1C1C] group-hover:bg-[#2E2E2E]">
            {tag}
          </span>
        ))}
        {tags.length > 3 && (
          <span className="text-[10px] text-gray-400 px-1.5 py-0.5 rounded border border-[#2E2E2E] group-hover:border-[#3E3E3E] bg-[#1C1C1C] group-hover:bg-[#2E2E2E]">
            +{tags.length - 3}
          </span>
        )}
      </div>
    )}
  </div>
);

// 工具卡片组件
const ToolCard: React.FC<ResourceCardProps & { category: string; }> = ({
  name, description, category, tags, updatedAt, onClick
}) => (
  <div className="p-3 border-b border-[#2E2E2E] hover:bg-[#2E2E2E] cursor-pointer group transition-colors" onClick={onClick}>
    <div className="flex items-start justify-between">
      <div className="flex gap-3">
        <div className="p-1.5 rounded-md bg-[#2E2E2E] group-hover:bg-[#3E3E3E] text-emerald-500">
          <Wrench size={16} />
        </div>
        <div>
          <h3 className="font-medium text-white text-sm">{name}</h3>
          <p className="text-xs text-gray-400 mt-1 line-clamp-1">{description}</p>
          <div className="flex gap-1.5 mt-2">
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#0D5F46]/20 text-emerald-400 border border-[#0D5F46]/20">
              {category}
            </span>
          </div>
        </div>
      </div>
      <Button variant="ghost" size="icon" className="h-7 w-7 rounded-md opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-white hover:bg-[#3E3E3E]" onClick={(e) => { e.stopPropagation(); }}>
        <MoreHorizontal size={14} />
      </Button>
    </div>
    {tags.length > 0 && (
      <div className="mt-3 flex flex-wrap gap-1 pl-8">
        {tags.slice(0, 3).map(tag => (
          <span key={tag} className="text-[10px] text-gray-400 px-1.5 py-0.5 rounded border border-[#2E2E2E] group-hover:border-[#3E3E3E] bg-[#1C1C1C] group-hover:bg-[#2E2E2E]">
            {tag}
          </span>
        ))}
        {tags.length > 3 && (
          <span className="text-[10px] text-gray-400 px-1.5 py-0.5 rounded border border-[#2E2E2E] group-hover:border-[#3E3E3E] bg-[#1C1C1C] group-hover:bg-[#2E2E2E]">
            +{tags.length - 3}
          </span>
        )}
      </div>
    )}
  </div>
);

// 代理卡片组件
const AgentCard: React.FC<ResourceCardProps & { status: string; model: string; }> = ({
  name, description, status, model, tags, updatedAt, onClick
}) => (
  <div className="p-3 border-b border-[#2E2E2E] hover:bg-[#2E2E2E] cursor-pointer group transition-colors" onClick={onClick}>
    <div className="flex items-start justify-between">
      <div className="flex gap-3">
        <div className="p-1.5 rounded-md bg-[#2E2E2E] group-hover:bg-[#3E3E3E] text-emerald-500">
          <UserCog size={16} />
        </div>
        <div>
          <h3 className="font-medium text-white text-sm">{name}</h3>
          <p className="text-xs text-gray-400 mt-1 line-clamp-1">{description}</p>
          <div className="flex gap-1.5 mt-2">
            <span className={`text-[10px] px-1.5 py-0.5 rounded border ${
              status === 'active' 
                ? 'bg-[#0D5F46]/20 text-emerald-400 border-[#0D5F46]/20' 
                : status === 'draft' 
                ? 'bg-[#854D0E]/20 text-amber-400 border-[#854D0E]/20' 
                : 'bg-[#2E2E2E] group-hover:bg-[#3E3E3E] text-gray-400 border-[#2E2E2E] group-hover:border-[#3E3E3E]'
            }`}>
              {status}
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#2C3494]/20 text-indigo-400 border border-[#2C3494]/20">
              {model}
            </span>
          </div>
        </div>
      </div>
      <Button variant="ghost" size="icon" className="h-7 w-7 rounded-md opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-white hover:bg-[#3E3E3E]" onClick={(e) => { e.stopPropagation(); }}>
        <MoreHorizontal size={14} />
      </Button>
    </div>
    {tags.length > 0 && (
      <div className="mt-3 flex flex-wrap gap-1 pl-8">
        {tags.slice(0, 3).map(tag => (
          <span key={tag} className="text-[10px] text-gray-400 px-1.5 py-0.5 rounded border border-[#2E2E2E] group-hover:border-[#3E3E3E] bg-[#1C1C1C] group-hover:bg-[#2E2E2E]">
            {tag}
          </span>
        ))}
        {tags.length > 3 && (
          <span className="text-[10px] text-gray-400 px-1.5 py-0.5 rounded border border-[#2E2E2E] group-hover:border-[#3E3E3E] bg-[#1C1C1C] group-hover:bg-[#2E2E2E]">
            +{tags.length - 3}
          </span>
        )}
      </div>
    )}
  </div>
);

// 工作流卡片组件
const WorkflowCard: React.FC<ResourceCardProps & { status: string; nodeCount: number; }> = ({
  name, description, status, nodeCount, tags, updatedAt, onClick
}) => (
  <div className="p-3 border-b border-[#2E2E2E] hover:bg-[#2E2E2E] cursor-pointer group transition-colors" onClick={onClick}>
    <div className="flex items-start justify-between">
      <div className="flex gap-3">
        <div className="p-1.5 rounded-md bg-[#2E2E2E] group-hover:bg-[#3E3E3E] text-emerald-500">
          <GitBranch size={16} />
        </div>
        <div>
          <h3 className="font-medium text-white text-sm">{name}</h3>
          <p className="text-xs text-gray-400 mt-1 line-clamp-1">{description}</p>
          <div className="flex gap-1.5 mt-2">
            <span className={`text-[10px] px-1.5 py-0.5 rounded border ${
              status === 'active' 
                ? 'bg-[#0D5F46]/20 text-emerald-400 border-[#0D5F46]/20' 
                : status === 'draft' 
                ? 'bg-[#854D0E]/20 text-amber-400 border-[#854D0E]/20' 
                : 'bg-[#2E2E2E] group-hover:bg-[#3E3E3E] text-gray-400 border-[#2E2E2E] group-hover:border-[#3E3E3E]'
            }`}>
              {status}
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#4C1D95]/20 text-purple-400 border border-[#4C1D95]/20">
              {nodeCount} nodes
            </span>
          </div>
        </div>
      </div>
      <Button variant="ghost" size="icon" className="h-7 w-7 rounded-md opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-white hover:bg-[#3E3E3E]" onClick={(e) => { e.stopPropagation(); }}>
        <MoreHorizontal size={14} />
      </Button>
    </div>
    {tags.length > 0 && (
      <div className="mt-3 flex flex-wrap gap-1 pl-8">
        {tags.slice(0, 3).map(tag => (
          <span key={tag} className="text-[10px] text-gray-400 px-1.5 py-0.5 rounded border border-[#2E2E2E] group-hover:border-[#3E3E3E] bg-[#1C1C1C] group-hover:bg-[#2E2E2E]">
            {tag}
          </span>
        ))}
        {tags.length > 3 && (
          <span className="text-[10px] text-gray-400 px-1.5 py-0.5 rounded border border-[#2E2E2E] group-hover:border-[#3E3E3E] bg-[#1C1C1C] group-hover:bg-[#2E2E2E]">
            +{tags.length - 3}
          </span>
        )}
      </div>
    )}
  </div>
);

// 空状态组件
const EmptyState: React.FC<{ type: ResourceType; onCreate?: () => void }> = ({ type, onCreate }) => {
  const typeMap = {
    model: { label: '模型', icon: <Brain size={28} /> },
    tool: { label: '工具', icon: <Wrench size={28} /> },
    agent: { label: '代理', icon: <UserCog size={28} /> },
    workflow: { label: '工作流', icon: <GitBranch size={28} /> }
  };

  return (
    <div className="flex flex-col items-center justify-center p-8 text-center">
      <div className="p-3 bg-[#2E2E2E] rounded-full mb-4 text-emerald-500">
        {typeMap[type].icon}
      </div>
      <h3 className="text-base font-medium mb-2 text-white">未找到{typeMap[type].label}</h3>
      <p className="text-gray-400 text-sm mb-6 max-w-md">
        创建您的第一个{typeMap[type].label}以开始构建AI应用
      </p>
      <Button className="h-8 gap-1 bg-emerald-600 hover:bg-emerald-700 text-white border-0" onClick={onCreate}>
        <Plus size={14} />
        <span className="text-xs font-medium">创建{typeMap[type].label}</span>
      </Button>
    </div>
  );
};

// 加载状态组件
const LoadingState: React.FC = () => (
  <div className="flex justify-center items-center h-[40vh]">
    <div className="animate-spin rounded-full h-8 w-8 border-2 border-emerald-500 border-r-transparent"></div>
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
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" size={14} />
          <Input 
            placeholder="搜索资源..." 
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="pl-9 h-8 text-sm bg-[#1C1C1C] border-[#2E2E2E] text-white focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500"
          />
        </div>
        <Button 
          type="submit" 
          variant="outline" 
          size="sm" 
          className="ml-2 h-8 text-xs border-[#2E2E2E] bg-[#1C1C1C] text-gray-300 hover:bg-[#2E2E2E] hover:text-white"
        >
          搜索
        </Button>
      </form>
      {onCreateNew && (
        <Button 
          onClick={onCreateNew}
          className="h-8 gap-1 bg-emerald-600 hover:bg-emerald-700 text-white border-0"
        >
          <Plus size={14} />
          <span className="text-xs font-medium">新建</span>
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
  <div className="bg-[#1C1C1C] border border-[#2E2E2E] rounded-md p-3">
    <div className="flex items-center justify-between mb-3">
      <div className="flex items-center gap-1.5">
        <Tag size={14} className="text-gray-400" />
        <span className="text-xs font-medium text-white">标签过滤</span>
      </div>
      <Button variant="ghost" size="sm" className="h-6 px-2 text-[10px] text-gray-400 hover:text-white hover:bg-[#2E2E2E]">
        清除
      </Button>
    </div>
    <div className="flex flex-wrap gap-1">
      {tags.length > 0 ? (
        tags.map((tag) => (
          <button
            key={tag}
            onClick={() => onTagSelect(tag)}
            className={`px-2 py-1 rounded-md text-xs border ${
              selectedTags.includes(tag)
                ? 'bg-emerald-900/30 text-emerald-400 border-emerald-900/50'
                : 'bg-[#1C1C1C] text-gray-400 border-[#2E2E2E] hover:bg-[#2E2E2E] hover:text-white'
            }`}
          >
            {tag}
          </button>
        ))
      ) : (
        <p className="text-gray-500 text-xs">无可用标签</p>
      )}
    </div>
  </div>
);

export const ResourceBrowser: React.FC = () => {
  const [activeTab, setActiveTab] = useState<ResourceType>('model');
  const [models, setModels] = useState<Model[]>([]);
  const [tools, setTools] = useState<Tool[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [showFilters, setShowFilters] = useState(false);

  // 加载资源
  useEffect(() => {
    const loadResources = async () => {
      setLoading(true);
      try {
        // 根据当前选择的类型加载资源
        switch (activeTab) {
          case 'model':
            const modelData = await getModels();
            setModels(modelData);
            // 提取标签
            const modelTags = Array.from(new Set(modelData.flatMap(model => model.tags)));
            setTags(modelTags);
            break;
          case 'tool':
            const toolData = await getTools();
            setTools(toolData);
            const toolTags = Array.from(new Set(toolData.flatMap(tool => tool.tags)));
            setTags(toolTags);
            break;
          case 'agent':
            const agentData = await getAgents();
            setAgents(agentData);
            const agentTags = Array.from(new Set(agentData.flatMap(agent => agent.tags)));
            setTags(agentTags);
            break;
          case 'workflow':
            const workflowData = await getWorkflows();
            setWorkflows(workflowData);
            const workflowTags = Array.from(new Set(workflowData.flatMap(workflow => workflow.tags)));
            setTags(workflowTags);
            break;
        }
        // 重置搜索和过滤
        setSearchQuery('');
        setSelectedTags([]);
      } catch (error) {
        console.error(`Error loading ${activeTab}s:`, error);
      } finally {
        setLoading(false);
      }
    };

    loadResources();
  }, [activeTab]);

  // 处理搜索
  const handleSearch = async (query: string) => {
    setSearchQuery(query);
    if (query.trim() === '') {
      // 如果搜索为空，回到常规加载
      const loadResources = async () => {
        switch (activeTab) {
          case 'model':
            const modelData = await getModels();
            setModels(modelData);
            break;
          case 'tool':
            const toolData = await getTools();
            setTools(toolData);
            break;
          case 'agent':
            const agentData = await getAgents();
            setAgents(agentData);
            break;
          case 'workflow':
            const workflowData = await getWorkflows();
            setWorkflows(workflowData);
            break;
        }
      };
      loadResources();
      return;
    }

    setLoading(true);
    try {
      const results = await searchResources(query, [activeTab]);
      switch (activeTab) {
        case 'model':
          setModels(results as Model[]);
          break;
        case 'tool':
          setTools(results as Tool[]);
          break;
        case 'agent':
          setAgents(results as Agent[]);
          break;
        case 'workflow':
          setWorkflows(results as Workflow[]);
          break;
      }
    } catch (error) {
      console.error('Search error:', error);
    } finally {
      setLoading(false);
    }
  };

  // 处理标签选择
  const handleTagSelect = (tag: string) => {
    setSelectedTags(prev => 
      prev.includes(tag) ? prev.filter(t => t !== tag) : [...prev, tag]
    );
  };

  // 按标签过滤资源
  const getFilteredResources = () => {
    if (selectedTags.length === 0) {
      switch (activeTab) {
        case 'model': return models;
        case 'tool': return tools;
        case 'agent': return agents;
        case 'workflow': return workflows;
        default: return [];
      }
    }

    switch (activeTab) {
      case 'model':
        return models.filter(model => 
          selectedTags.every(tag => model.tags.includes(tag))
        );
      case 'tool':
        return tools.filter(tool => 
          selectedTags.every(tag => tool.tags.includes(tag))
        );
      case 'agent':
        return agents.filter(agent => 
          selectedTags.every(tag => agent.tags.includes(tag))
        );
      case 'workflow':
        return workflows.filter(workflow => 
          selectedTags.every(tag => workflow.tags.includes(tag))
        );
      default:
        return [];
    }
  };

  // 处理创建新资源
  const handleCreateResource = (type: ResourceType) => {
    // 实现创建不同类型资源的功能
    console.log(`Creating new ${type}`);
  };

  // 处理选择资源
  const handleResourceSelect = (id: string, type: ResourceType) => {
    // 实现选择资源详情的功能
    console.log(`Selected ${type} with id ${id}`);
  };

  const filteredResources = getFilteredResources();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-medium text-white">资源库</h1>
          <p className="text-sm text-gray-400 mt-1">管理和使用您的AI资源</p>
        </div>
        <div className="flex gap-2">
          <Button 
            variant="outline" 
            size="sm" 
            className="h-8 px-2 text-xs border-[#2E2E2E] bg-[#1C1C1C] text-gray-300 hover:bg-[#2E2E2E] hover:text-white"
            onClick={() => setShowFilters(!showFilters)}
          >
            <SlidersHorizontal size={14} className="mr-1.5" />
            筛选
            <ChevronDown size={14} className={`ml-1.5 transition-transform ${showFilters ? 'rotate-180' : ''}`} />
          </Button>
          <Button 
            className="h-8 gap-1 bg-emerald-600 hover:bg-emerald-700 text-white border-0"
            onClick={() => handleCreateResource(activeTab)}
          >
            <Plus size={14} />
            <span className="text-xs font-medium">创建{
              activeTab === 'model' ? '模型' :
              activeTab === 'tool' ? '工具' :
              activeTab === 'agent' ? '代理' : '工作流'
            }</span>
          </Button>
        </div>
      </div>
      
      <div className="flex flex-col md:flex-row gap-4">
        <div className={`${showFilters ? 'block' : 'hidden md:block'} md:w-56`}>
          <ResourceFilters 
            tags={tags} 
            selectedTags={selectedTags} 
            onTagSelect={handleTagSelect} 
          />
          
          <div className="mt-4 bg-[#1C1C1C] border border-[#2E2E2E] rounded-md overflow-hidden">
            <div className="p-3 border-b border-[#2E2E2E]">
              <span className="text-xs font-medium text-white">资源类型</span>
            </div>
            <div className="flex flex-col">
              <button 
                className={`flex items-center gap-2 p-2.5 text-left text-xs ${activeTab === 'model' ? 'bg-[#2E2E2E] text-white' : 'text-gray-400 hover:bg-[#2E2E2E] hover:text-white'}`}
                onClick={() => setActiveTab('model')}
              >
                <Brain size={14} />
                <span>模型</span>
              </button>
              <button 
                className={`flex items-center gap-2 p-2.5 text-left text-xs ${activeTab === 'tool' ? 'bg-[#2E2E2E] text-white' : 'text-gray-400 hover:bg-[#2E2E2E] hover:text-white'}`}
                onClick={() => setActiveTab('tool')}
              >
                <Wrench size={14} />
                <span>工具</span>
              </button>
              <button 
                className={`flex items-center gap-2 p-2.5 text-left text-xs ${activeTab === 'agent' ? 'bg-[#2E2E2E] text-white' : 'text-gray-400 hover:bg-[#2E2E2E] hover:text-white'}`}
                onClick={() => setActiveTab('agent')}
              >
                <UserCog size={14} />
                <span>代理</span>
              </button>
              <button 
                className={`flex items-center gap-2 p-2.5 text-left text-xs ${activeTab === 'workflow' ? 'bg-[#2E2E2E] text-white' : 'text-gray-400 hover:bg-[#2E2E2E] hover:text-white'}`}
                onClick={() => setActiveTab('workflow')}
              >
                <GitBranch size={14} />
                <span>工作流</span>
              </button>
            </div>
          </div>
        </div>
        
        <div className="flex-1">
          <div className="mb-4">
            <ResourceSearch 
              onSearch={handleSearch} 
              onCreateNew={() => handleCreateResource(activeTab)}
            />
          </div>
          
          <div className="bg-[#1C1C1C] border border-[#2E2E2E] rounded-md overflow-hidden">
            <div className="flex justify-between items-center p-3 border-b border-[#2E2E2E]">
              <div className="flex items-center gap-1.5">
                {activeTab === 'model' && <Brain size={14} className="text-emerald-500" />}
                {activeTab === 'tool' && <Wrench size={14} className="text-emerald-500" />}
                {activeTab === 'agent' && <UserCog size={14} className="text-emerald-500" />}
                {activeTab === 'workflow' && <GitBranch size={14} className="text-emerald-500" />}
                <h2 className="text-sm font-medium text-white">
                  {activeTab === 'model' ? '模型' : 
                   activeTab === 'tool' ? '工具' : 
                   activeTab === 'agent' ? '代理' : 
                   '工作流'}
                </h2>
                {filteredResources && filteredResources.length > 0 && (
                  <span className="ml-1.5 px-1.5 py-0.5 rounded-full text-[10px] bg-[#2E2E2E] text-gray-400">
                    {filteredResources.length}
                  </span>
                )}
              </div>
              <div className="flex gap-2">
                <Button variant="ghost" size="sm" className="h-7 text-xs text-gray-400 hover:text-white hover:bg-[#2E2E2E]">
                  <FolderOpen size={14} className="mr-1.5" />
                  导入
                </Button>
                <Button variant="ghost" size="sm" className="h-7 text-xs text-gray-400 hover:text-white hover:bg-[#2E2E2E]">
                  <Filter size={14} className="mr-1.5" />
                  排序
                </Button>
              </div>
            </div>
            
            {loading ? (
              <LoadingState />
            ) : filteredResources && filteredResources.length > 0 ? (
              <div>
                {activeTab === 'model' && (
                  (filteredResources as Model[]).map((model) => (
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
                  ))
                )}
                {activeTab === 'tool' && (
                  (filteredResources as Tool[]).map((tool) => (
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
                  ))
                )}
                {activeTab === 'agent' && (
                  (filteredResources as Agent[]).map((agent) => (
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
                  ))
                )}
                {activeTab === 'workflow' && (
                  (filteredResources as Workflow[]).map((workflow) => (
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
                  ))
                )}
              </div>
            ) : (
              <EmptyState 
                type={activeTab} 
                onCreate={() => handleCreateResource(activeTab)}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}; 