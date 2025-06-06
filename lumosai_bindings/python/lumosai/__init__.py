"""
Lumos.ai Python绑定

高性能AI Agent框架的Python接口，基于Rust核心实现。

基本使用:
    >>> from lumosai import Agent, tools
    >>> 
    >>> # 快速创建Agent
    >>> agent = Agent.quick("assistant", "你是一个AI助手") \\
    ...     .model("deepseek-chat") \\
    ...     .tools([tools.web_search(), tools.calculator()]) \\
    ...     .build()
    >>> 
    >>> # 生成响应
    >>> response = await agent.generate("帮我搜索最新的AI新闻")
    >>> print(response.content)

高级使用:
    >>> # 使用构建器模式
    >>> agent = Agent.builder() \\
    ...     .name("research_assistant") \\
    ...     .instructions("你是一个专业的研究助手") \\
    ...     .model("gpt-4") \\
    ...     .tools([
    ...         tools.web_search(),
    ...         tools.file_reader(),
    ...         tools.calculator(),
    ...     ]) \\
    ...     .build()
    >>> 
    >>> # 异步生成
    >>> response = await agent.generate_async("分析这个文件的内容")
"""

from typing import List, Optional, Dict, Any, Union
import asyncio
import warnings

# 导入Rust核心绑定
try:
    from lumosai._core import (
        Agent as _Agent,
        AgentBuilder as _AgentBuilder,
        Tool as _Tool,
        Response as _Response,
        LumosError,
        quick_agent as _quick_agent,
        create_agent_builder as _create_agent_builder,
    )
except ImportError as e:
    raise ImportError(
        "Failed to import Lumos.ai core bindings. "
        "Please ensure the package is properly installed. "
        f"Error: {e}"
    ) from e

# 版本信息
__version__ = "0.1.0"
__author__ = "Lumos.ai Team"
__email__ = "team@lumosai.com"
__license__ = "MIT OR Apache-2.0"

# 导出的公共API
__all__ = [
    "Agent",
    "AgentBuilder", 
    "Tool",
    "Response",
    "LumosError",
    "tools",
    "__version__",
]


class Agent:
    """
    Lumos.ai AI Agent
    
    高性能的AI Agent，支持工具调用、异步执行和流式响应。
    
    Examples:
        >>> # 快速创建
        >>> agent = Agent.quick("assistant", "你是一个AI助手").build()
        >>> 
        >>> # 生成响应
        >>> response = agent.generate("Hello, world!")
        >>> print(response.content)
    """
    
    def __init__(self, inner_agent: _Agent):
        """初始化Agent"""
        self._inner = inner_agent
    
    def generate(self, input_text: str) -> "Response":
        """
        生成响应（同步）
        
        Args:
            input_text: 输入文本
            
        Returns:
            Response: 生成的响应
            
        Raises:
            LumosError: 生成失败时抛出
        """
        return Response(self._inner.generate(input_text))
    
    async def generate_async(self, input_text: str) -> "Response":
        """
        生成响应（异步）
        
        Args:
            input_text: 输入文本
            
        Returns:
            Response: 生成的响应
            
        Raises:
            LumosError: 生成失败时抛出
        """
        response = await self._inner.generate_async(input_text)
        return Response(response)
    
    def get_config(self) -> Dict[str, Any]:
        """
        获取Agent配置
        
        Returns:
            Dict[str, Any]: 配置信息
        """
        return self._inner.get_config()
    
    @classmethod
    def quick(cls, name: str, instructions: str) -> "AgentBuilder":
        """
        快速创建Agent构建器
        
        Args:
            name: Agent名称
            instructions: 指令
            
        Returns:
            AgentBuilder: Agent构建器
        """
        return AgentBuilder(_quick_agent(name, instructions))
    
    @classmethod
    def builder(cls) -> "AgentBuilder":
        """
        创建Agent构建器
        
        Returns:
            AgentBuilder: Agent构建器
        """
        return AgentBuilder(_create_agent_builder())
    
    def __str__(self) -> str:
        return "Lumos.ai Agent"
    
    def __repr__(self) -> str:
        return "Agent()"


class AgentBuilder:
    """
    Agent构建器
    
    使用构建器模式创建和配置Agent。
    
    Examples:
        >>> builder = Agent.builder() \\
        ...     .name("assistant") \\
        ...     .instructions("你是一个AI助手") \\
        ...     .model("gpt-4") \\
        ...     .tools([tools.web_search()])
        >>> 
        >>> agent = builder.build()
    """
    
    def __init__(self, inner_builder: _AgentBuilder):
        """初始化AgentBuilder"""
        self._inner = inner_builder
    
    def name(self, name: str) -> "AgentBuilder":
        """
        设置Agent名称
        
        Args:
            name: Agent名称
            
        Returns:
            AgentBuilder: 返回自身以支持链式调用
        """
        self._inner.name(name)
        return self
    
    def instructions(self, instructions: str) -> "AgentBuilder":
        """
        设置Agent指令
        
        Args:
            instructions: 指令文本
            
        Returns:
            AgentBuilder: 返回自身以支持链式调用
        """
        self._inner.instructions(instructions)
        return self
    
    def model(self, model: str) -> "AgentBuilder":
        """
        设置模型
        
        Args:
            model: 模型名称
            
        Returns:
            AgentBuilder: 返回自身以支持链式调用
        """
        self._inner.model(model)
        return self
    
    def tool(self, tool: "Tool") -> "AgentBuilder":
        """
        添加工具
        
        Args:
            tool: 工具实例
            
        Returns:
            AgentBuilder: 返回自身以支持链式调用
        """
        self._inner.tool(tool._inner)
        return self
    
    def tools(self, tools: List["Tool"]) -> "AgentBuilder":
        """
        添加多个工具
        
        Args:
            tools: 工具列表
            
        Returns:
            AgentBuilder: 返回自身以支持链式调用
        """
        tool_list = [tool._inner for tool in tools]
        self._inner.tools(tool_list)
        return self
    
    def build(self) -> Agent:
        """
        构建Agent
        
        Returns:
            Agent: 构建的Agent实例
            
        Raises:
            LumosError: 构建失败时抛出
        """
        return Agent(self._inner.build())
    
    async def build_async(self) -> Agent:
        """
        异步构建Agent
        
        Returns:
            Agent: 构建的Agent实例
            
        Raises:
            LumosError: 构建失败时抛出
        """
        inner_agent = await self._inner.build_async()
        return Agent(inner_agent)
    
    def __str__(self) -> str:
        return "Lumos.ai AgentBuilder"
    
    def __repr__(self) -> str:
        return "AgentBuilder()"


class Tool:
    """
    Lumos.ai工具
    
    封装各种功能的工具，如网络搜索、文件操作、数学计算等。
    
    Examples:
        >>> tool = tools.web_search()
        >>> metadata = tool.metadata()
        >>> print(metadata['name'])  # 'web_search'
        >>> 
        >>> result = tool.execute(query="Python教程")
        >>> print(result['success'])  # True
    """
    
    def __init__(self, inner_tool: _Tool):
        """初始化Tool"""
        self._inner = inner_tool
    
    def metadata(self) -> Dict[str, Any]:
        """
        获取工具元数据
        
        Returns:
            Dict[str, Any]: 工具元数据
        """
        return self._inner.metadata()
    
    def execute(self, **kwargs) -> Dict[str, Any]:
        """
        执行工具
        
        Args:
            **kwargs: 工具参数
            
        Returns:
            Dict[str, Any]: 执行结果
            
        Raises:
            LumosError: 执行失败时抛出
        """
        return self._inner.execute(kwargs)
    
    def __str__(self) -> str:
        metadata = self.metadata()
        return f"Tool({metadata.get('name', 'unknown')})"
    
    def __repr__(self) -> str:
        metadata = self.metadata()
        return f"Tool(name='{metadata.get('name', 'unknown')}', type='{metadata.get('tool_type', 'unknown')}')"


class Response:
    """
    Agent响应
    
    包含生成的内容、元数据和工具调用结果。
    
    Examples:
        >>> response = agent.generate("Hello")
        >>> print(response.content)
        >>> print(response.response_type)
        >>> 
        >>> if response.has_error:
        ...     print(f"Error: {response.error}")
    """
    
    def __init__(self, inner_response: _Response):
        """初始化Response"""
        self._inner = inner_response
    
    @property
    def content(self) -> str:
        """响应内容"""
        return self._inner.content
    
    @property
    def response_type(self) -> str:
        """响应类型"""
        return self._inner.response_type
    
    @property
    def metadata(self) -> Dict[str, Any]:
        """响应元数据"""
        return self._inner.metadata
    
    @property
    def tool_calls(self) -> List[Dict[str, Any]]:
        """工具调用结果"""
        return self._inner.tool_calls
    
    @property
    def has_error(self) -> bool:
        """是否有错误"""
        return self._inner.has_error
    
    @property
    def error(self) -> Optional[str]:
        """错误信息"""
        return self._inner.error
    
    def __str__(self) -> str:
        return self.content
    
    def __repr__(self) -> str:
        content_preview = self.content[:50] + "..." if len(self.content) > 50 else self.content
        return f"Response(content='{content_preview}', type={self.response_type})"


# 工具模块
class _ToolsModule:
    """工具模块，提供各种预定义工具"""
    
    def __getattr__(self, name: str):
        """动态导入工具"""
        try:
            from lumosai._core import tools as _tools
            tool_func = getattr(_tools, name)
            return lambda: Tool(tool_func())
        except AttributeError:
            raise AttributeError(f"Tool '{name}' not found")
    
    def web_search(self) -> Tool:
        """Web搜索工具"""
        from lumosai._core.tools import web_search
        return Tool(web_search())
    
    def http_request(self) -> Tool:
        """HTTP请求工具"""
        from lumosai._core.tools import http_request
        return Tool(http_request())
    
    def file_reader(self) -> Tool:
        """文件读取工具"""
        from lumosai._core.tools import file_reader
        return Tool(file_reader())
    
    def file_writer(self) -> Tool:
        """文件写入工具"""
        from lumosai._core.tools import file_writer
        return Tool(file_writer())
    
    def calculator(self) -> Tool:
        """计算器工具"""
        from lumosai._core.tools import calculator
        return Tool(calculator())
    
    def json_processor(self) -> Tool:
        """JSON处理工具"""
        from lumosai._core.tools import json_processor
        return Tool(json_processor())
    
    def csv_processor(self) -> Tool:
        """CSV处理工具"""
        from lumosai._core.tools import csv_processor
        return Tool(csv_processor())


# 创建工具模块实例
tools = _ToolsModule()


# 兼容性检查
def _check_python_version():
    """检查Python版本兼容性"""
    import sys
    if sys.version_info < (3, 8):
        warnings.warn(
            "Lumos.ai requires Python 3.8 or later. "
            f"Current version: {sys.version_info.major}.{sys.version_info.minor}",
            UserWarning,
            stacklevel=2
        )


# 初始化检查
_check_python_version()
