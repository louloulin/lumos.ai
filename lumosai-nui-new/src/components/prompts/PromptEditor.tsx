import React, { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { Switch } from '../ui/switch';
import { Badge } from '../ui/badge';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '../ui/tooltip';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '../ui/dialog';
import { 
  Play, Save, Plus, X, Tag, Clock, Info, Copy, 
  PanelLeft, Trash2, Zap, Share2, Sparkles, MessageSquare
} from 'lucide-react';
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { promptsService, PromptTemplate, PromptVariable, extractVariablesFromContent, validatePromptTemplate } from '../../services/prompts';

// 代码编辑器组件，支持语法高亮
import CodeMirror from '@uiw/react-codemirror';
import { markdown } from '@codemirror/lang-markdown';
import { okaidia } from '@uiw/codemirror-theme-okaidia';

// 变量管理组件
const VariableManager: React.FC<{
  variables: PromptVariable[];
  onChange: (variables: PromptVariable[]) => void;
  extractedVars: string[];
}> = ({ variables, onChange, extractedVars }) => {
  // 添加一个新变量
  const addVariable = () => {
    const newVar: PromptVariable = {
      name: '',
      description: '',
      required: true,
      type: 'string'
    };
    onChange([...variables, newVar]);
  };

  // 更新一个变量
  const updateVariable = (index: number, field: keyof PromptVariable, value: any) => {
    const updatedVars = [...variables];
    updatedVars[index] = { ...updatedVars[index], [field]: value };
    onChange(updatedVars);
  };

  // 删除一个变量
  const removeVariable = (index: number) => {
    const updatedVars = [...variables];
    updatedVars.splice(index, 1);
    onChange(updatedVars);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium">模板变量</h3>
        <Button 
          onClick={addVariable} 
          size="sm" 
          variant="outline"
          className="bg-emerald-600/10 text-emerald-500 border-emerald-600/20 hover:bg-emerald-600/20"
        >
          <Plus size={16} className="mr-1" /> 添加变量
        </Button>
      </div>
      
      {variables.length === 0 ? (
        <div className="text-center p-4 bg-gray-800/50 rounded-md border border-gray-700">
          <p className="text-gray-400">没有定义变量。点击"添加变量"按钮创建变量。</p>
        </div>
      ) : (
        <div className="space-y-4">
          {variables.map((variable, index) => (
            <Card key={index} className="bg-gray-900/70 border-gray-700">
              <CardHeader className="pb-2 pt-4 px-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Input
                      value={variable.name}
                      onChange={(e) => updateVariable(index, 'name', e.target.value)}
                      placeholder="变量名称"
                      className="max-w-[200px] bg-gray-800 border-gray-700"
                    />
                    {extractedVars.includes(variable.name) ? (
                      <Badge className="bg-emerald-600/40 text-emerald-300">已使用</Badge>
                    ) : variable.name ? (
                      <Badge className="bg-amber-600/20 text-amber-300">未使用</Badge>
                    ) : null}
                  </div>
                  <Button 
                    variant="ghost" 
                    size="icon" 
                    onClick={() => removeVariable(index)}
                    className="h-8 w-8 text-gray-400 hover:text-white hover:bg-red-900/20"
                  >
                    <Trash2 size={16} />
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="space-y-3 pt-0 px-4">
                <div>
                  <Input
                    value={variable.description}
                    onChange={(e) => updateVariable(index, 'description', e.target.value)}
                    placeholder="变量描述"
                    className="bg-gray-800 border-gray-700"
                  />
                </div>
                <div className="flex flex-wrap gap-4">
                  <div className="flex items-center space-x-2">
                    <label className="text-sm text-gray-400">类型:</label>
                    <Select
                      value={variable.type}
                      onValueChange={(value) => updateVariable(index, 'type', value)}
                    >
                      <SelectTrigger className="w-[130px] bg-gray-800 border-gray-700">
                        <SelectValue placeholder="选择类型" />
                      </SelectTrigger>
                      <SelectContent className="bg-gray-800 border-gray-700">
                        <SelectItem value="string">文本</SelectItem>
                        <SelectItem value="number">数字</SelectItem>
                        <SelectItem value="boolean">布尔值</SelectItem>
                        <SelectItem value="array">数组</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="flex items-center space-x-2">
                    <label className="text-sm text-gray-400">必填:</label>
                    <Switch
                      checked={variable.required}
                      onCheckedChange={(checked) => updateVariable(index, 'required', checked)}
                      className="data-[state=checked]:bg-emerald-500"
                    />
                  </div>
                </div>
                {variable.required === false && (
                  <div>
                    <Input
                      value={variable.defaultValue || ''}
                      onChange={(e) => updateVariable(index, 'defaultValue', e.target.value)}
                      placeholder="默认值"
                      className="bg-gray-800 border-gray-700"
                    />
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      )}
      
      <div className="mt-2">
        <h4 className="text-sm font-medium text-gray-400 mb-2">模板中检测到的变量:</h4>
        <div className="flex flex-wrap gap-2">
          {extractedVars.length === 0 ? (
            <p className="text-sm text-gray-500">未检测到变量。使用 {{变量名}} 格式在模板中添加变量。</p>
          ) : (
            extractedVars.map((varName) => {
              const isDefined = variables.some(v => v.name === varName);
              return (
                <Badge 
                  key={varName} 
                  className={isDefined 
                    ? "bg-emerald-600/40 text-emerald-300" 
                    : "bg-red-500/40 text-red-300 cursor-pointer"
                  }
                  onClick={() => {
                    if (!isDefined) {
                      onChange([...variables, {
                        name: varName,
                        description: '',
                        required: true,
                        type: 'string'
                      }]);
                    }
                  }}
                >
                  {varName}
                  {!isDefined && (
                    <Plus size={12} className="ml-1" />
                  )}
                </Badge>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};

// 提示模板测试组件
const PromptTester: React.FC<{
  template: PromptTemplate | null;
  variables: PromptVariable[];
}> = ({ template, variables }) => {
  const [variableValues, setVariableValues] = useState<Record<string, any>>({});
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<{
    output: string;
    tokenUsage?: { prompt: number; completion: number; total: number };
    executionTime?: number;
    modelUsed?: string;
  } | null>(null);
  const [selectedModel, setSelectedModel] = useState(template?.model || 'gpt-4');

  useEffect(() => {
    // 初始化变量值，使用默认值（如果有）
    const initialValues: Record<string, any> = {};
    variables.forEach(v => {
      initialValues[v.name] = v.defaultValue || '';
    });
    setVariableValues(initialValues);
  }, [variables]);

  const runPrompt = async () => {
    if (!template) return;
    
    setIsRunning(true);
    setResult(null);
    
    try {
      const response = await promptsService.executePrompt({
        templateId: template.id,
        variables: variableValues,
        model: selectedModel,
      });
      
      setResult({
        output: response.output,
        tokenUsage: response.tokenUsage,
        executionTime: response.executionTime,
        modelUsed: response.modelUsed,
      });
    } catch (error) {
      setResult({
        output: `错误: ${(error as Error).message}`,
      });
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <h3 className="text-lg font-medium">输入变量</h3>
        
        {variables.length === 0 ? (
          <div className="text-center p-4 bg-gray-800/50 rounded-md border border-gray-700">
            <p className="text-gray-400">该模板没有定义变量。请先在变量页添加变量。</p>
          </div>
        ) : (
          <div className="space-y-3">
            {variables.map((v) => (
              <div key={v.name} className="space-y-1">
                <div className="flex items-center space-x-2">
                  <label className="text-sm font-medium">
                    {v.name}
                    {v.required && <span className="text-red-400 ml-1">*</span>}
                  </label>
                  {v.description && (
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Info size={14} className="text-gray-400" />
                        </TooltipTrigger>
                        <TooltipContent className="bg-gray-800 border-gray-700">
                          <p>{v.description}</p>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  )}
                </div>
                
                {v.type === 'boolean' ? (
                  <div className="flex items-center">
                    <Switch
                      checked={variableValues[v.name] === true}
                      onCheckedChange={(checked) => 
                        setVariableValues({...variableValues, [v.name]: checked})
                      }
                      className="data-[state=checked]:bg-emerald-500"
                    />
                    <span className="ml-2 text-sm text-gray-400">
                      {variableValues[v.name] === true ? '是' : '否'}
                    </span>
                  </div>
                ) : v.type === 'number' ? (
                  <Input
                    type="number"
                    value={variableValues[v.name] || ''}
                    onChange={(e) => 
                      setVariableValues({
                        ...variableValues, 
                        [v.name]: e.target.value === '' ? '' : Number(e.target.value)
                      })
                    }
                    placeholder={v.description}
                    className="bg-gray-800 border-gray-700"
                  />
                ) : (
                  <Textarea
                    value={variableValues[v.name] || ''}
                    onChange={(e) => 
                      setVariableValues({...variableValues, [v.name]: e.target.value})
                    }
                    placeholder={v.description}
                    className="min-h-[80px] bg-gray-800 border-gray-700"
                  />
                )}
              </div>
            ))}
          </div>
        )}
      </div>
      
      <div className="flex items-center space-x-4">
        <div>
          <label className="text-sm text-gray-400 mb-1 block">模型</label>
          <Select
            value={selectedModel}
            onValueChange={setSelectedModel}
          >
            <SelectTrigger className="w-[180px] bg-gray-800 border-gray-700">
              <SelectValue placeholder="选择模型" />
            </SelectTrigger>
            <SelectContent className="bg-gray-800 border-gray-700">
              <SelectItem value="gpt-4">GPT-4</SelectItem>
              <SelectItem value="gpt-3.5-turbo">GPT-3.5 Turbo</SelectItem>
              <SelectItem value="llama2-70b">Llama 2 (70B)</SelectItem>
              <SelectItem value="claude-3-opus">Claude 3 Opus</SelectItem>
              <SelectItem value="local-model">本地模型</SelectItem>
            </SelectContent>
          </Select>
        </div>
        
        <Button 
          onClick={runPrompt} 
          disabled={isRunning || variables.length === 0}
          className="bg-emerald-600 hover:bg-emerald-700 text-white ml-auto"
        >
          {isRunning ? (
            <>
              <div className="animate-spin mr-2 h-4 w-4 border-2 border-white border-opacity-20 border-t-white rounded-full" />
              执行中...
            </>
          ) : (
            <>
              <Play size={16} className="mr-2" />
              执行提示
            </>
          )}
        </Button>
      </div>
      
      {result && (
        <div className="space-y-4 mt-8">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium">结果输出</h3>
            <Button 
              variant="outline"
              size="sm"
              className="bg-gray-800/60 border-gray-700 hover:bg-gray-700/60"
              onClick={() => {
                if (result.output) {
                  navigator.clipboard.writeText(result.output);
                }
              }}
            >
              <Copy size={14} className="mr-1" />
              复制
            </Button>
          </div>
          
          <Card className="bg-gray-900/70 border-gray-700">
            <CardContent className="p-4">
              <div className="whitespace-pre-wrap font-mono text-sm bg-black/30 p-4 rounded-md border border-gray-800">
                {result.output}
              </div>
            </CardContent>
            {(result.tokenUsage || result.executionTime || result.modelUsed) && (
              <CardFooter className="px-4 py-3 bg-black/20 border-t border-gray-800 flex items-center justify-between text-xs text-gray-400">
                <div className="flex items-center space-x-4">
                  {result.modelUsed && (
                    <div className="flex items-center">
                      <Sparkles size={12} className="mr-1 text-blue-400" />
                      <span>模型: {result.modelUsed}</span>
                    </div>
                  )}
                  {result.tokenUsage && (
                    <div className="flex items-center">
                      <MessageSquare size={12} className="mr-1 text-emerald-400" />
                      <span>Token: {result.tokenUsage.total} ({result.tokenUsage.prompt}+{result.tokenUsage.completion})</span>
                    </div>
                  )}
                </div>
                {result.executionTime && (
                  <div className="flex items-center">
                    <Clock size={12} className="mr-1 text-amber-400" />
                    <span>执行时间: {(result.executionTime / 1000).toFixed(2)}s</span>
                  </div>
                )}
              </CardFooter>
            )}
          </Card>
        </div>
      )}
    </div>
  );
};

// 主提示编辑器组件
const PromptEditor: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState('edit');
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  
  const [template, setTemplate] = useState<PromptTemplate | null>(null);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [content, setContent] = useState('');
  const [variables, setVariables] = useState<PromptVariable[]>([]);
  const [tags, setTags] = useState<string[]>([]);
  const [isPublic, setIsPublic] = useState(false);
  const [category, setCategory] = useState('');
  const [newTag, setNewTag] = useState('');

  const [extractedVars, setExtractedVars] = useState<string[]>([]);
  const [validationErrors, setValidationErrors] = useState<string[]>([]);
  
  // 加载提示模板
  useEffect(() => {
    if (id === 'new') return;

    const loadTemplate = async () => {
      setIsLoading(true);
      try {
        const template = await promptsService.getPromptTemplateById(id!);
        if (template) {
          setTemplate(template);
          setName(template.name);
          setDescription(template.description);
          setContent(template.content);
          setVariables(template.variables);
          setTags(template.tags);
          setIsPublic(template.isPublic);
          setCategory(template.category || '');
          
          // 提取变量
          const extractedVars = extractVariablesFromContent(template.content);
          setExtractedVars(extractedVars);
        }
      } catch (error) {
        console.error('Failed to load template:', error);
      } finally {
        setIsLoading(false);
      }
    };

    loadTemplate();
  }, [id]);

  // 当内容变化时更新提取的变量
  useEffect(() => {
    const extracted = extractVariablesFromContent(content);
    setExtractedVars(extracted);
    
    // 运行验证
    const { errors } = validatePromptTemplate({
      name,
      description,
      content,
      variables
    });
    setValidationErrors(errors);
  }, [content, name, description, variables]);

  // 保存提示模板
  const saveTemplate = async () => {
    setIsSaving(true);
    try {
      const templateData = {
        name,
        description,
        content,
        variables,
        tags,
        isPublic,
        category: category || undefined
      };
      
      if (id === 'new') {
        const newTemplate = await promptsService.createPromptTemplate(
          templateData,
          'user-1' // 假设当前用户ID
        );
        setTemplate(newTemplate);
        navigate(`/prompts/${newTemplate.id}`);
      } else {
        const updatedTemplate = await promptsService.updatePromptTemplate(id!, templateData);
        setTemplate(updatedTemplate);
      }
    } catch (error) {
      console.error('Failed to save template:', error);
    } finally {
      setIsSaving(false);
    }
  };

  // 删除提示模板
  const deleteTemplate = async () => {
    if (!id || id === 'new') return;
    
    try {
      await promptsService.deletePromptTemplate(id);
      navigate('/prompts');
    } catch (error) {
      console.error('Failed to delete template:', error);
    }
  };

  // 添加标签
  const addTag = () => {
    if (newTag && !tags.includes(newTag)) {
      setTags([...tags, newTag]);
      setNewTag('');
    }
  };

  // 删除标签
  const removeTag = (tag: string) => {
    setTags(tags.filter(t => t !== tag));
  };

  // 模板内容编辑器属性
  const editorTheme = React.useMemo(() => okaidia, []);

  // 加载中状态
  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full p-8">
        <div className="flex flex-col items-center space-y-4">
          <div className="animate-spin h-8 w-8 border-4 border-emerald-500 border-opacity-20 border-t-emerald-500 rounded-full" />
          <p className="text-gray-400">加载提示模板...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-6 max-w-7xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold mb-1">
            {id === 'new' ? '创建新提示模板' : name || '编辑提示模板'}
          </h1>
          {template && (
            <div className="flex items-center space-x-2 text-sm text-gray-400">
              <span>版本 {template.version}</span>
              <span>•</span>
              <span>更新于 {new Date(template.updatedAt).toLocaleString()}</span>
            </div>
          )}
        </div>
        
        <div className="flex items-center space-x-3">
          <Button 
            variant="outline" 
            disabled={id === 'new'}
            onClick={() => setShowDeleteDialog(true)}
            className="border-red-800/30 text-red-500 hover:bg-red-900/20 hover:text-red-400"
          >
            <Trash2 size={16} className="mr-2" />
            删除
          </Button>
          
          <Button 
            onClick={saveTemplate} 
            disabled={isSaving || validationErrors.length > 0}
            className="bg-emerald-600 hover:bg-emerald-700 text-white"
          >
            {isSaving ? (
              <>
                <div className="animate-spin mr-2 h-4 w-4 border-2 border-white border-opacity-20 border-t-white rounded-full" />
                保存中...
              </>
            ) : (
              <>
                <Save size={16} className="mr-2" />
                保存模板
              </>
            )}
          </Button>
        </div>
      </div>
      
      {validationErrors.length > 0 && (
        <div className="mb-6 p-4 border border-red-800/50 bg-red-900/20 rounded-md">
          <h3 className="text-red-400 font-medium mb-2">存在以下错误需要修复：</h3>
          <ul className="list-disc list-inside text-sm text-red-300">
            {validationErrors.map((error, i) => (
              <li key={i}>{error}</li>
            ))}
          </ul>
        </div>
      )}
      
      <Card className="bg-gray-900/70 border-gray-700 shadow-lg mb-6">
        <CardHeader className="pb-2">
          <CardTitle>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="模板名称"
              className="text-lg font-medium bg-gray-800 border-gray-700"
            />
          </CardTitle>
          <CardDescription>
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="模板描述 (用途、适用场景等)"
              className="mt-2 resize-none bg-gray-800 border-gray-700"
              rows={2}
            />
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap gap-3 items-center">
            <div className="flex items-center space-x-2">
              <label className="text-sm text-gray-400">公开:</label>
              <Switch
                checked={isPublic}
                onCheckedChange={setIsPublic}
                className="data-[state=checked]:bg-emerald-500"
              />
              <span className="text-sm text-gray-400">
                {isPublic ? '是' : '否'}
              </span>
            </div>
            
            <div className="flex-1 min-w-[200px]">
              <Select
                value={category}
                onValueChange={setCategory}
              >
                <SelectTrigger className="bg-gray-800 border-gray-700">
                  <SelectValue placeholder="选择类别" />
                </SelectTrigger>
                <SelectContent className="bg-gray-800 border-gray-700">
                  <SelectItem value="general">通用</SelectItem>
                  <SelectItem value="development">开发</SelectItem>
                  <SelectItem value="business">商业</SelectItem>
                  <SelectItem value="analysis">分析</SelectItem>
                  <SelectItem value="dialogue">对话</SelectItem>
                  <SelectItem value="creative">创意</SelectItem>
                  <SelectItem value="academic">学术</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          
          <div>
            <label className="text-sm text-gray-400 block mb-1">标签</label>
            <div className="flex flex-wrap gap-2 items-center mb-2">
              {tags.map(tag => (
                <Badge key={tag} className="bg-blue-900/50 text-blue-300 hover:bg-blue-800/50">
                  {tag}
                  <X
                    size={14}
                    className="ml-1 cursor-pointer text-blue-300 hover:text-blue-100"
                    onClick={() => removeTag(tag)}
                  />
                </Badge>
              ))}
              <div className="flex items-center">
                <Input
                  value={newTag}
                  onChange={(e) => setNewTag(e.target.value)}
                  placeholder="添加标签..."
                  className="max-w-[150px] h-8 bg-gray-800 border-gray-700 text-sm"
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      addTag();
                    }
                  }}
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={addTag}
                  disabled={!newTag}
                  className="h-8 w-8 ml-1 text-gray-400"
                >
                  <Plus size={16} />
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Tabs value={activeTab} onValueChange={setActiveTab} className="mb-6">
        <TabsList className="bg-gray-800 border-gray-700">
          <TabsTrigger value="edit" className="data-[state=active]:bg-gray-700">
            模板编辑
          </TabsTrigger>
          <TabsTrigger value="variables" className="data-[state=active]:bg-gray-700">
            变量管理
          </TabsTrigger>
          <TabsTrigger value="test" className="data-[state=active]:bg-gray-700">
            测试运行
          </TabsTrigger>
        </TabsList>
        
        <TabsContent value="edit" className="mt-4">
          <Card className="bg-gray-900/70 border-gray-700 shadow-lg">
            <CardContent className="p-4">
              <div className="mb-2">
                <p className="text-sm text-gray-400 mb-2">
                  编写提示模板，使用 {{变量名}} 格式添加变量占位符
                </p>
              </div>
              <div className="border rounded-md border-gray-700 bg-black/30">
                <CodeMirror
                  value={content}
                  height="400px"
                  theme={editorTheme}
                  extensions={[markdown()]}
                  onChange={setContent}
                  className="text-base"
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>
        
        <TabsContent value="variables" className="mt-4">
          <Card className="bg-gray-900/70 border-gray-700 shadow-lg">
            <CardContent className="p-4">
              <VariableManager
                variables={variables}
                onChange={setVariables}
                extractedVars={extractedVars}
              />
            </CardContent>
          </Card>
        </TabsContent>
        
        <TabsContent value="test" className="mt-4">
          <Card className="bg-gray-900/70 border-gray-700 shadow-lg">
            <CardContent className="p-4">
              <PromptTester
                template={template}
                variables={variables}
              />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
      
      <Dialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <DialogContent className="bg-gray-900 border-gray-700">
          <DialogHeader>
            <DialogTitle>确认删除</DialogTitle>
            <DialogDescription>
              您确定要删除此提示模板吗？此操作无法撤销。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setShowDeleteDialog(false)}
              className="bg-gray-800 hover:bg-gray-700 border-gray-700"
            >
              取消
            </Button>
            <Button
              onClick={deleteTemplate}
              className="bg-red-600 hover:bg-red-700 text-white"
            >
              删除
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default PromptEditor; 