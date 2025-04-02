import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Badge } from '../ui/badge';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table';
import { 
  Search, ArrowUpDown, ChevronDown, Clock, MessageSquare, 
  Zap, Download, Eye, Info, Filter, Calendar, MoreVertical, FileText
} from 'lucide-react';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '../ui/dialog';
import { Popover, PopoverContent, PopoverTrigger } from '../ui/popover';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '../ui/dropdown-menu';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { 
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue
} from '../ui/select';

// 执行历史记录类型
type PromptExecution = {
  id: string;
  templateId: string;
  templateName: string;
  executedAt: string;
  executionTime: number;
  modelUsed: string;
  tokenUsage: {
    prompt: number;
    completion: number;
    total: number;
  };
  status: 'success' | 'error';
  errorMessage?: string;
  userId: string;
  input: Record<string, any>;
  output: string;
};

// 模拟数据
const MOCK_EXECUTIONS: PromptExecution[] = [
  {
    id: 'exec-1',
    templateId: 'pt-1',
    templateName: '通用知识问答',
    executedAt: '2023-05-10T14:30:00Z',
    executionTime: 1250,
    modelUsed: 'gpt-4',
    tokenUsage: {
      prompt: 45,
      completion: 120,
      total: 165
    },
    status: 'success',
    userId: 'user-1',
    input: {
      question: '什么是量子计算？'
    },
    output: '量子计算是一种利用量子力学现象（如叠加和纠缠）来执行计算的计算模型。传统计算机使用二进制位来表示信息，而量子计算机使用量子比特（qubits）。由于量子比特可以同时处于多个状态，理论上量子计算机可以比传统计算机更快地解决某些类型的问题。'
  },
  {
    id: 'exec-2',
    templateId: 'pt-2',
    templateName: '代码解释器',
    executedAt: '2023-05-11T09:15:00Z',
    executionTime: 2100,
    modelUsed: 'gpt-4',
    tokenUsage: {
      prompt: 230,
      completion: 345,
      total: 575
    },
    status: 'success',
    userId: 'user-1',
    input: {
      language: 'javascript',
      code: 'function fibonacci(n) {\n  if (n <= 1) return n;\n  return fibonacci(n-1) + fibonacci(n-2);\n}'
    },
    output: '这段代码定义了一个名为fibonacci的递归函数，它计算斐波那契数列的第n个数。斐波那契数列是一个整数序列，其中每个数是前两个数的和，从0和1开始。\n\n函数首先检查输入n是否小于或等于1，如果是则直接返回n（这是递归的基本情况）。如果n大于1，函数会调用自身两次，计算n-1和n-2的斐波那契数，然后返回它们的和。\n\n这个实现简单明了，但对于较大的n值效率很低，因为它会导致大量的重复计算。更高效的实现可以使用动态规划或记忆化。'
  },
  {
    id: 'exec-3',
    templateId: 'pt-3',
    templateName: '情感分析',
    executedAt: '2023-05-12T16:45:00Z',
    executionTime: 950,
    modelUsed: 'gpt-3.5-turbo',
    tokenUsage: {
      prompt: 120,
      completion: 80,
      total: 200
    },
    status: 'success',
    userId: 'user-1',
    input: {
      text: '这家餐厅的食物非常美味，但服务态度令人失望，等了很久才上菜。'
    },
    output: '情感分析评分：3/5\n\n该文本表达了混合的情感。积极方面，用户认为食物"非常美味"，这是明显的正面评价。然而，负面方面包括对服务态度的失望和长时间等待上菜的抱怨。综合来看，这是一个中性偏向略微负面的评价，因为虽然食物质量好，但整体用餐体验被服务问题所影响。'
  },
  {
    id: 'exec-4',
    templateId: 'pt-4',
    templateName: '营销文案生成器',
    executedAt: '2023-05-15T11:20:00Z',
    executionTime: 1800,
    modelUsed: 'gpt-4',
    tokenUsage: {
      prompt: 280,
      completion: 420,
      total: 700
    },
    status: 'success',
    userId: 'user-1',
    input: {
      product: 'EcoFresh智能空气净化器',
      features: '- 采用HEPA滤网，过滤效率达99.97%\n- 智能感应技术，自动调节净化速度\n- 超静音运行，低至20分贝\n- 手机APP远程控制\n- 节能设计，比普通净化器节省30%电量',
      tone: 'friendly',
      audience: '年轻家庭和注重健康的专业人士'
    },
    output: '呼吸健康的新选择，为家人带来纯净呵护！\n\nEcoFresh智能空气净化器不只是一台家电，更是家庭健康的守护者。高效HEPA滤网可捕捉99.97%的微小颗粒，从花粉到PM2.5，让您和家人远离空气中的有害物质。\n\n我们懂得您忙碌的生活，EcoFresh的智能感应系统会自动检测空气质量，悄悄调整净化速度，无需您操心。超静音设计确保即使婴儿熟睡时也不会被打扰，只有20分贝的运行声，比轻声低语还要安静！\n\n通过手机APP，您可以在通勤途中或办公室随时查看家中空气质量并远程控制，回家即可享受新鲜空气。更棒的是，节能设计让您的电费账单也能减重30%，对环境和钱包都更友好！\n\n现代生活很复杂，但呼吸健康的空气不必如此。EcoFresh，让每一次呼吸都充满纯净的幸福感！'
  },
  {
    id: 'exec-5',
    templateId: 'pt-1',
    templateName: '通用知识问答',
    executedAt: '2023-05-16T15:10:00Z',
    executionTime: 1050,
    modelUsed: 'gpt-4',
    tokenUsage: {
      prompt: 50,
      completion: 280,
      total: 330
    },
    status: 'success',
    userId: 'user-1',
    input: {
      question: '什么是大模型微调？'
    },
    output: '大模型微调（Large Language Model Fine-tuning）是指在预训练好的大型语言模型（如GPT-4、LLaMA等）基础上，使用特定领域的数据集或针对特定任务的数据进行进一步训练，以使模型更好地适应特定应用场景的过程。\n\n与从头训练模型相比，微调具有以下优势：\n1. 资源需求更低：只需调整部分参数而非全部参数\n2. 训练时间更短：大模型已经学习了语言的基本结构和知识\n3. 样本需求更少：利用迁移学习，只需相对较少的任务特定样本\n\n常见的微调方法包括：\n- 全参数微调（Full Fine-tuning）：调整模型所有参数\n- 参数高效微调（PEFT）：如LoRA、Prefix-tuning等，只调整少量参数\n- 指令微调（Instruction Tuning）：训练模型遵循特定格式的指令\n- RLHF（Reinforcement Learning from Human Feedback）：使用人类反馈的强化学习\n\n微调对于构建垂直领域的AI应用（如医疗、法律、金融等）或开发特定功能（如代码生成、内容审核）的定制化模型尤为重要。'
  },
  {
    id: 'exec-6',
    templateId: 'pt-5',
    templateName: '对话系统提示',
    executedAt: '2023-05-17T10:25:00Z',
    executionTime: 680,
    modelUsed: 'gpt-3.5-turbo',
    tokenUsage: {
      prompt: 150,
      completion: 95,
      total: 245
    },
    status: 'error',
    errorMessage: '执行超时，服务端30秒内未返回结果',
    userId: 'user-1',
    input: {
      character: 'TechGuru',
      expertise: '计算机科学和人工智能',
      tone: 'educational yet casual',
      behavior: 'break down complex concepts into simple analogies',
      input: '解释一下神经网络的工作原理'
    },
    output: ''
  },
  {
    id: 'exec-7',
    templateId: 'pt-2',
    templateName: '代码解释器',
    executedAt: '2023-05-18T14:00:00Z',
    executionTime: 1750,
    modelUsed: 'claude-3-opus',
    tokenUsage: {
      prompt: 320,
      completion: 410,
      total: 730
    },
    status: 'success',
    userId: 'user-1',
    input: {
      language: 'python',
      code: 'def quicksort(arr):\n    if len(arr) <= 1:\n        return arr\n    pivot = arr[len(arr) // 2]\n    left = [x for x in arr if x < pivot]\n    middle = [x for x in arr if x == pivot]\n    right = [x for x in arr if x > pivot]\n    return quicksort(left) + middle + quicksort(right)'
    },
    output: '这段代码实现了快速排序（Quicksort）算法，一种高效的分治排序算法。\n\n以下是代码的工作原理：\n\n1. 首先检查数组长度，如果小于或等于1，直接返回数组（基本情况）。\n\n2. 选择一个"枢轴"（pivot）元素，这里选择的是数组的中间元素。\n\n3. 将数组分成三部分：\n   - left：所有小于枢轴的元素\n   - middle：所有等于枢轴的元素\n   - right：所有大于枢轴的元素\n\n4. 递归地对left和right部分应用quicksort。\n\n5. 最后，将排序后的left、middle和right数组连接起来返回。\n\n这个实现使用了Python的列表推导式来高效地分割数组。快速排序的平均时间复杂度是O(n log n)，但在最坏情况下可能退化为O(n²)。这种实现直观且易于理解，但不是原地排序，因此需要额外的空间。在实际应用中，如果空间效率重要，可能需要考虑原地快速排序实现。'
  },
  {
    id: 'exec-8',
    templateId: 'pt-3',
    templateName: '情感分析',
    executedAt: '2023-05-19T09:40:00Z',
    executionTime: 880,
    modelUsed: 'gpt-3.5-turbo',
    tokenUsage: {
      prompt: 80,
      completion: 110,
      total: 190
    },
    status: 'success',
    userId: 'user-1',
    input: {
      text: '我非常满意这款产品的质量和性能，超出了我的预期，绝对值得购买！'
    },
    output: '情感分析评分：5/5\n\n这段文本表达了非常积极的情感。用户使用了多个强烈的正面词汇，如"非常满意"、"超出了我的预期"和"绝对值得购买"，表明对产品的高度认可。文本中没有任何负面或中性的评价成分，整体传达了对产品质量和性能的完全满意。这是一个典型的五星好评。'
  }
];

// 模拟查询API
const queryExecutionHistory = async (
  userId: string,
  filters?: {
    templateId?: string;
    startDate?: string;
    endDate?: string;
    status?: 'success' | 'error' | 'all';
    model?: string;
    searchText?: string;
  }
): Promise<PromptExecution[]> => {
  // 模拟API延迟
  await new Promise(resolve => setTimeout(resolve, 800));
  
  // 筛选执行记录
  let results = MOCK_EXECUTIONS.filter(exec => exec.userId === userId);
  
  if (filters) {
    if (filters.templateId) {
      results = results.filter(exec => exec.templateId === filters.templateId);
    }
    
    if (filters.startDate) {
      const startDate = new Date(filters.startDate);
      results = results.filter(exec => new Date(exec.executedAt) >= startDate);
    }
    
    if (filters.endDate) {
      const endDate = new Date(filters.endDate);
      results = results.filter(exec => new Date(exec.executedAt) <= endDate);
    }
    
    if (filters.status && filters.status !== 'all') {
      results = results.filter(exec => exec.status === filters.status);
    }
    
    if (filters.model) {
      results = results.filter(exec => exec.modelUsed === filters.model);
    }
    
    if (filters.searchText) {
      const searchLower = filters.searchText.toLowerCase();
      results = results.filter(exec => 
        exec.templateName.toLowerCase().includes(searchLower) ||
        exec.output.toLowerCase().includes(searchLower) ||
        JSON.stringify(exec.input).toLowerCase().includes(searchLower)
      );
    }
  }
  
  // 按执行时间降序排序
  return results.sort((a, b) => 
    new Date(b.executedAt).getTime() - new Date(a.executedAt).getTime()
  );
};

// 执行详情对话框
const ExecutionDetailsDialog: React.FC<{
  execution: PromptExecution | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}> = ({ execution, open, onOpenChange }) => {
  if (!execution) return null;
  
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="bg-gray-900 border-gray-700 max-w-4xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>执行详情</DialogTitle>
          <DialogDescription>
            执行ID: {execution.id}
          </DialogDescription>
        </DialogHeader>
        
        <div className="space-y-6 py-2">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <h4 className="text-sm font-medium text-gray-400 mb-1">提示模板</h4>
              <p>{execution.templateName}</p>
            </div>
            <div>
              <h4 className="text-sm font-medium text-gray-400 mb-1">执行时间</h4>
              <p>{new Date(execution.executedAt).toLocaleString()}</p>
            </div>
            <div>
              <h4 className="text-sm font-medium text-gray-400 mb-1">模型</h4>
              <p>{execution.modelUsed}</p>
            </div>
            <div>
              <h4 className="text-sm font-medium text-gray-400 mb-1">状态</h4>
              <div>
                {execution.status === 'success' ? (
                  <Badge className="bg-emerald-600/40 text-emerald-300">成功</Badge>
                ) : (
                  <Badge className="bg-red-600/40 text-red-300">失败</Badge>
                )}
              </div>
            </div>
            <div>
              <h4 className="text-sm font-medium text-gray-400 mb-1">Token 使用量</h4>
              <p>{execution.tokenUsage.total} ({execution.tokenUsage.prompt}+{execution.tokenUsage.completion})</p>
            </div>
            <div>
              <h4 className="text-sm font-medium text-gray-400 mb-1">执行耗时</h4>
              <p>{(execution.executionTime / 1000).toFixed(2)}s</p>
            </div>
          </div>
          
          <Tabs defaultValue="input">
            <TabsList className="bg-gray-800 border-gray-700">
              <TabsTrigger value="input" className="data-[state=active]:bg-gray-700">输入参数</TabsTrigger>
              <TabsTrigger value="output" className="data-[state=active]:bg-gray-700">输出结果</TabsTrigger>
            </TabsList>
            
            <TabsContent value="input" className="mt-4">
              <Card className="bg-gray-900/70 border-gray-700">
                <CardContent className="p-4">
                  <pre className="whitespace-pre-wrap font-mono text-sm bg-black/30 p-4 rounded-md border border-gray-800 max-h-[300px] overflow-y-auto">
                    {JSON.stringify(execution.input, null, 2)}
                  </pre>
                </CardContent>
              </Card>
            </TabsContent>
            
            <TabsContent value="output" className="mt-4">
              <Card className="bg-gray-900/70 border-gray-700">
                <CardContent className="p-4">
                  {execution.status === 'error' ? (
                    <div className="p-4 bg-red-900/20 border border-red-800/50 rounded-md text-red-300">
                      <h4 className="font-medium mb-2">执行错误</h4>
                      <p>{execution.errorMessage}</p>
                    </div>
                  ) : (
                    <pre className="whitespace-pre-wrap font-mono text-sm bg-black/30 p-4 rounded-md border border-gray-800 max-h-[300px] overflow-y-auto">
                      {execution.output}
                    </pre>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </div>
      </DialogContent>
    </Dialog>
  );
};

// 主历史记录组件
const PromptHistory: React.FC = () => {
  const navigate = useNavigate();
  const [executions, setExecutions] = useState<PromptExecution[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  
  // 过滤状态
  const [selectedStatus, setSelectedStatus] = useState<'all' | 'success' | 'error'>('all');
  const [selectedModel, setSelectedModel] = useState('');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  
  // 详情对话框状态
  const [selectedExecution, setSelectedExecution] = useState<PromptExecution | null>(null);
  const [detailsDialogOpen, setDetailsDialogOpen] = useState(false);
  
  // 可用的模型列表
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  
  // 加载执行历史
  useEffect(() => {
    const loadHistory = async () => {
      setIsLoading(true);
      try {
        // 假设当前用户ID
        const userId = 'user-1';
        
        const executions = await queryExecutionHistory(userId, {
          status: selectedStatus === 'all' ? undefined : selectedStatus,
          model: selectedModel || undefined,
          startDate: startDate || undefined,
          endDate: endDate || undefined,
          searchText: searchQuery || undefined
        });
        
        setExecutions(executions);
        
        // 提取可用的模型
        const models = new Set<string>();
        executions.forEach(exec => models.add(exec.modelUsed));
        setAvailableModels(Array.from(models));
      } catch (error) {
        console.error('Failed to load execution history:', error);
      } finally {
        setIsLoading(false);
      }
    };

    loadHistory();
  }, [selectedStatus, selectedModel, startDate, endDate, searchQuery]);
  
  // 查看执行详情
  const viewExecutionDetails = (execution: PromptExecution) => {
    setSelectedExecution(execution);
    setDetailsDialogOpen(true);
  };
  
  // 清除所有过滤器
  const clearFilters = () => {
    setSearchQuery('');
    setSelectedStatus('all');
    setSelectedModel('');
    setStartDate('');
    setEndDate('');
  };
  
  // 格式化日期
  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };
  
  // 检查是否应用了任何过滤器
  const hasActiveFilters = 
    searchQuery !== '' || 
    selectedStatus !== 'all' || 
    selectedModel !== '' || 
    startDate !== '' || 
    endDate !== '';

  return (
    <div className="container mx-auto p-6 max-w-7xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold mb-1">提示执行历史</h1>
          <p className="text-gray-400">查看和分析提示模板的历史执行记录</p>
        </div>
        
        <Button 
          onClick={() => navigate('/prompts')}
          className="bg-emerald-600 hover:bg-emerald-700 text-white"
        >
          查看提示模板
        </Button>
      </div>
      
      <Card className="bg-gray-900/70 border-gray-700 mb-6">
        <CardContent className="p-4">
          <div className="flex flex-col md:flex-row gap-4">
            <div className="relative flex-1">
              <Search size={18} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
              <Input
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="搜索执行历史..."
                className="pl-10 bg-gray-800 border-gray-700 focus:ring-emerald-500/30 focus:border-emerald-500/60"
              />
            </div>
            
            <div className="flex flex-wrap items-center gap-3">
              <Popover>
                <PopoverTrigger asChild>
                  <Button 
                    variant="outline"
                    className={`bg-gray-800 border-gray-700 ${startDate || endDate ? 'text-emerald-400 border-emerald-500/50' : ''}`}
                  >
                    <Calendar size={16} className="mr-2" />
                    {startDate || endDate ? '日期已筛选' : '日期范围'}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-80 p-3 bg-gray-800 border-gray-700">
                  <div className="space-y-4">
                    <h4 className="font-medium text-sm">按日期筛选</h4>
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="text-sm text-gray-400 mb-1 block">开始日期</label>
                        <Input
                          type="date"
                          value={startDate}
                          onChange={(e) => setStartDate(e.target.value)}
                          className="bg-gray-900 border-gray-700"
                        />
                      </div>
                      <div>
                        <label className="text-sm text-gray-400 mb-1 block">结束日期</label>
                        <Input
                          type="date"
                          value={endDate}
                          onChange={(e) => setEndDate(e.target.value)}
                          className="bg-gray-900 border-gray-700"
                        />
                      </div>
                    </div>
                  </div>
                </PopoverContent>
              </Popover>
              
              <Select
                value={selectedStatus}
                onValueChange={(value: any) => setSelectedStatus(value)}
              >
                <SelectTrigger className={`w-32 bg-gray-800 border-gray-700 ${selectedStatus !== 'all' ? 'text-emerald-400 border-emerald-500/50' : ''}`}>
                  <SelectValue placeholder="状态" />
                </SelectTrigger>
                <SelectContent className="bg-gray-800 border-gray-700">
                  <SelectItem value="all">全部状态</SelectItem>
                  <SelectItem value="success">成功</SelectItem>
                  <SelectItem value="error">失败</SelectItem>
                </SelectContent>
              </Select>
              
              <Select
                value={selectedModel}
                onValueChange={setSelectedModel}
              >
                <SelectTrigger className={`w-36 bg-gray-800 border-gray-700 ${selectedModel ? 'text-emerald-400 border-emerald-500/50' : ''}`}>
                  <SelectValue placeholder="模型" />
                </SelectTrigger>
                <SelectContent className="bg-gray-800 border-gray-700">
                  <SelectItem value="">全部模型</SelectItem>
                  {availableModels.map(model => (
                    <SelectItem key={model} value={model}>{model}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
              
              {hasActiveFilters && (
                <Button 
                  variant="ghost"
                  onClick={clearFilters}
                  className="text-gray-400 hover:text-white"
                >
                  清除
                </Button>
              )}
              
              <Button 
                variant="outline"
                className="bg-gray-800 border-gray-700 ml-auto"
                onClick={() => {
                  // 这里应实现导出功能
                  alert('导出功能将在未来版本中实现');
                }}
              >
                <Download size={16} className="mr-2" />
                导出
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
      
      {isLoading ? (
        <div className="flex items-center justify-center h-64">
          <div className="flex flex-col items-center space-y-4">
            <div className="animate-spin h-8 w-8 border-4 border-emerald-500 border-opacity-20 border-t-emerald-500 rounded-full" />
            <p className="text-gray-400">加载执行历史...</p>
          </div>
        </div>
      ) : executions.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <div className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center mb-4">
            <Clock size={24} className="text-gray-400" />
          </div>
          <h3 className="text-xl font-semibold mb-2">没有执行记录</h3>
          <p className="text-gray-400 mb-6 max-w-md">
            {hasActiveFilters 
              ? '没有符合当前筛选条件的执行记录。尝试调整过滤器查看更多结果。'
              : '您尚未执行任何提示模板。尝试运行一个提示模板来生成执行记录。'}
          </p>
          {hasActiveFilters ? (
            <Button onClick={clearFilters} className="bg-gray-700 hover:bg-gray-600">
              清除所有过滤器
            </Button>
          ) : (
            <Button onClick={() => navigate('/prompts')} className="bg-emerald-600 hover:bg-emerald-700">
              查看提示模板
            </Button>
          )}
        </div>
      ) : (
        <Card className="bg-gray-900/70 border-gray-700">
          <CardHeader className="pb-0">
            <CardTitle className="text-lg">
              执行记录 ({executions.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="p-0 overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow className="border-gray-700 hover:bg-transparent">
                  <TableHead className="w-[250px]">提示模板</TableHead>
                  <TableHead className="w-[180px]">执行时间</TableHead>
                  <TableHead>模型</TableHead>
                  <TableHead>Token用量</TableHead>
                  <TableHead>执行耗时</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {executions.map((execution) => (
                  <TableRow key={execution.id} className="border-gray-700 hover:bg-gray-800/50">
                    <TableCell className="font-medium">{execution.templateName}</TableCell>
                    <TableCell>{formatDate(execution.executedAt)}</TableCell>
                    <TableCell>{execution.modelUsed}</TableCell>
                    <TableCell>{execution.tokenUsage.total}</TableCell>
                    <TableCell>{(execution.executionTime / 1000).toFixed(2)}s</TableCell>
                    <TableCell>
                      {execution.status === 'success' ? (
                        <Badge className="bg-emerald-600/40 text-emerald-300">成功</Badge>
                      ) : (
                        <Badge className="bg-red-600/40 text-red-300">失败</Badge>
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button 
                            variant="ghost" 
                            size="icon"
                            className="h-8 w-8 text-gray-400 hover:text-white hover:bg-gray-700/50"
                          >
                            <MoreVertical size={16} />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end" className="bg-gray-800 border-gray-700">
                          <DropdownMenuItem 
                            className="cursor-pointer hover:bg-gray-700"
                            onClick={() => viewExecutionDetails(execution)}
                          >
                            <Eye size={14} className="mr-2" />
                            查看详情
                          </DropdownMenuItem>
                          <DropdownMenuItem 
                            className="cursor-pointer hover:bg-gray-700"
                            onClick={() => navigate(`/prompts/${execution.templateId}`)}
                          >
                            <FileText size={14} className="mr-2" />
                            编辑模板
                          </DropdownMenuItem>
                          <DropdownMenuItem 
                            className="cursor-pointer hover:bg-gray-700"
                            onClick={() => navigate(`/prompts/${execution.templateId}?tab=test`)}
                          >
                            <Zap size={14} className="mr-2" />
                            运行模板
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}
      
      <ExecutionDetailsDialog
        execution={selectedExecution}
        open={detailsDialogOpen}
        onOpenChange={setDetailsDialogOpen}
      />
    </div>
  );
};

export default PromptHistory; 