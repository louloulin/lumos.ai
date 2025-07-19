// LumosAI Web UI JavaScript
class LumosAIUI {
    constructor() {
        this.currentPage = 'dashboard';
        this.sidebarOpen = true;
        this.websocket = null;
        this.agents = [];
        this.workflows = [];
        this.tools = [];
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.setupWebSocket();
        this.loadInitialData();
        this.startPerformanceMonitoring();
    }

    setupEventListeners() {
        // 侧边栏切换
        document.getElementById('sidebar-toggle').addEventListener('click', () => {
            this.toggleSidebar();
        });

        // 导航菜单
        document.querySelectorAll('.nav-item').forEach(item => {
            item.addEventListener('click', (e) => {
                e.preventDefault();
                const page = e.currentTarget.getAttribute('href').substring(1);
                this.navigateTo(page);
            });
        });

        // 快速操作按钮
        this.setupQuickActions();
    }

    setupQuickActions() {
        // 创建新Agent
        document.querySelector('button:has(.fa-plus)').addEventListener('click', () => {
            this.showCreateAgentDialog();
        });

        // 运行工作流
        document.querySelector('button:has(.fa-play)').addEventListener('click', () => {
            this.showRunWorkflowDialog();
        });

        // 添加工具
        document.querySelector('button:has(.fa-wrench)').addEventListener('click', () => {
            this.showAddToolDialog();
        });
    }

    setupWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        this.websocket = new WebSocket(wsUrl);
        
        this.websocket.onopen = () => {
            console.log('WebSocket连接已建立');
            this.updateConnectionStatus(true);
        };

        this.websocket.onmessage = (event) => {
            const data = JSON.parse(event.data);
            this.handleWebSocketMessage(data);
        };

        this.websocket.onclose = () => {
            console.log('WebSocket连接已关闭');
            this.updateConnectionStatus(false);
            // 尝试重连
            setTimeout(() => this.setupWebSocket(), 5000);
        };

        this.websocket.onerror = (error) => {
            console.error('WebSocket错误:', error);
        };
    }

    handleWebSocketMessage(data) {
        switch (data.type) {
            case 'agent_status':
                this.updateAgentStatus(data.payload);
                break;
            case 'workflow_update':
                this.updateWorkflowStatus(data.payload);
                break;
            case 'performance_metrics':
                this.updatePerformanceMetrics(data.payload);
                break;
            case 'log_entry':
                this.addLogEntry(data.payload);
                break;
            default:
                console.log('未知消息类型:', data.type);
        }
    }

    toggleSidebar() {
        this.sidebarOpen = !this.sidebarOpen;
        const sidebar = document.getElementById('sidebar');
        const mainContent = document.querySelector('.ml-64');
        
        if (this.sidebarOpen) {
            sidebar.style.transform = 'translateX(0)';
            mainContent.style.marginLeft = '16rem';
        } else {
            sidebar.style.transform = 'translateX(-100%)';
            mainContent.style.marginLeft = '0';
        }
    }

    navigateTo(page) {
        // 隐藏所有页面
        document.querySelectorAll('.page-content').forEach(p => {
            p.classList.add('hidden');
        });

        // 显示目标页面
        const targetPage = document.getElementById(`${page}-page`);
        if (targetPage) {
            targetPage.classList.remove('hidden');
        }

        // 更新导航状态
        document.querySelectorAll('.nav-item').forEach(item => {
            item.classList.remove('active', 'bg-blue-50', 'text-blue-600');
        });
        
        const activeItem = document.querySelector(`[href="#${page}"]`);
        if (activeItem) {
            activeItem.classList.add('active', 'bg-blue-50', 'text-blue-600');
        }

        // 更新页面标题
        const titles = {
            dashboard: '仪表板',
            agents: 'Agent管理',
            workflows: '工作流',
            tools: '工具管理',
            monitoring: '性能监控',
            logs: '日志查看'
        };
        
        document.getElementById('page-title').textContent = titles[page] || page;
        this.currentPage = page;

        // 加载页面特定数据
        this.loadPageData(page);
    }

    loadPageData(page) {
        switch (page) {
            case 'agents':
                this.loadAgents();
                break;
            case 'workflows':
                this.loadWorkflows();
                break;
            case 'tools':
                this.loadTools();
                break;
            case 'monitoring':
                this.loadMonitoringData();
                break;
            case 'logs':
                this.loadLogs();
                break;
        }
    }

    async loadInitialData() {
        try {
            // 加载基础统计数据
            const stats = await this.fetchAPI('/api/stats');
            this.updateDashboardStats(stats);

            // 加载最近活动
            const activities = await this.fetchAPI('/api/activities');
            this.updateRecentActivities(activities);

        } catch (error) {
            console.error('加载初始数据失败:', error);
        }
    }

    async loadAgents() {
        try {
            const agents = await this.fetchAPI('/api/agents');
            this.agents = agents;
            this.renderAgents();
        } catch (error) {
            console.error('加载Agent失败:', error);
        }
    }

    renderAgents() {
        const grid = document.getElementById('agents-grid');
        grid.innerHTML = '';

        this.agents.forEach(agent => {
            const card = this.createAgentCard(agent);
            grid.appendChild(card);
        });
    }

    createAgentCard(agent) {
        const card = document.createElement('div');
        card.className = 'bg-white border rounded-lg p-4 hover:shadow-lg transition-shadow';
        
        const statusColor = agent.status === 'running' ? 'green' : 
                           agent.status === 'stopped' ? 'red' : 'yellow';
        
        card.innerHTML = `
            <div class="flex items-center justify-between mb-3">
                <h4 class="font-semibold text-gray-800">${agent.name}</h4>
                <div class="w-3 h-3 bg-${statusColor}-500 rounded-full"></div>
            </div>
            <p class="text-sm text-gray-600 mb-3">${agent.description}</p>
            <div class="flex items-center justify-between text-xs text-gray-500">
                <span>模型: ${agent.model}</span>
                <span>工具: ${agent.tools_count}</span>
            </div>
            <div class="mt-3 flex space-x-2">
                <button class="flex-1 bg-blue-500 text-white py-1 px-2 rounded text-xs hover:bg-blue-600">
                    测试
                </button>
                <button class="flex-1 bg-gray-500 text-white py-1 px-2 rounded text-xs hover:bg-gray-600">
                    编辑
                </button>
            </div>
        `;

        return card;
    }

    updateDashboardStats(stats) {
        document.getElementById('active-agents').textContent = stats.active_agents || 0;
        document.getElementById('running-workflows').textContent = stats.running_workflows || 0;
        document.getElementById('available-tools').textContent = stats.available_tools || 0;
        document.getElementById('avg-response').textContent = `${stats.avg_response_time || 0}s`;
    }

    updateRecentActivities(activities) {
        const container = document.getElementById('recent-activities');
        container.innerHTML = '';

        activities.forEach(activity => {
            const item = document.createElement('div');
            item.className = 'flex items-center p-3 bg-gray-50 rounded-lg';
            
            const colorMap = {
                info: 'blue',
                success: 'green',
                warning: 'yellow',
                error: 'red'
            };
            
            const color = colorMap[activity.type] || 'gray';
            
            item.innerHTML = `
                <div class="w-2 h-2 bg-${color}-500 rounded-full mr-3"></div>
                <div class="flex-1">
                    <p class="text-sm font-medium text-gray-900">${activity.message}</p>
                    <p class="text-xs text-gray-500">${this.formatTime(activity.timestamp)}</p>
                </div>
            `;
            
            container.appendChild(item);
        });
    }

    updateConnectionStatus(connected) {
        const statusElement = document.querySelector('.flex.items-center.text-sm.text-gray-600');
        const dot = statusElement.querySelector('.w-2.h-2');
        const text = statusElement.lastChild;
        
        if (connected) {
            dot.className = 'w-2 h-2 bg-green-500 rounded-full mr-2';
            text.textContent = '服务器运行中';
        } else {
            dot.className = 'w-2 h-2 bg-red-500 rounded-full mr-2';
            text.textContent = '连接断开';
        }
    }

    startPerformanceMonitoring() {
        setInterval(() => {
            if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
                this.websocket.send(JSON.stringify({
                    type: 'request_metrics'
                }));
            }
        }, 5000); // 每5秒请求一次性能指标
    }

    async fetchAPI(endpoint) {
        const response = await fetch(endpoint);
        if (!response.ok) {
            throw new Error(`API请求失败: ${response.status}`);
        }
        return response.json();
    }

    formatTime(timestamp) {
        const now = new Date();
        const time = new Date(timestamp);
        const diff = Math.floor((now - time) / 1000);
        
        if (diff < 60) return `${diff}秒前`;
        if (diff < 3600) return `${Math.floor(diff / 60)}分钟前`;
        if (diff < 86400) return `${Math.floor(diff / 3600)}小时前`;
        return `${Math.floor(diff / 86400)}天前`;
    }

    showCreateAgentDialog() {
        // 显示创建Agent对话框
        alert('创建Agent功能开发中...');
    }

    showRunWorkflowDialog() {
        // 显示运行工作流对话框
        alert('运行工作流功能开发中...');
    }

    showAddToolDialog() {
        // 显示添加工具对话框
        alert('添加工具功能开发中...');
    }
}

// 初始化应用
document.addEventListener('DOMContentLoaded', () => {
    window.lumosAI = new LumosAIUI();
});

// 模拟数据（开发阶段使用）
if (window.location.hostname === 'localhost') {
    // 模拟API响应
    const mockAPI = {
        '/api/stats': {
            active_agents: 3,
            running_workflows: 2,
            available_tools: 12,
            avg_response_time: 1.2
        },
        '/api/activities': [
            {
                type: 'success',
                message: '研究助手Agent已启动',
                timestamp: Date.now() - 120000
            },
            {
                type: 'info',
                message: '数据分析工作流执行完成',
                timestamp: Date.now() - 300000
            },
            {
                type: 'warning',
                message: '新工具已添加: 天气查询',
                timestamp: Date.now() - 600000
            }
        ],
        '/api/agents': [
            {
                name: '研究助手',
                description: '专业的研究和信息收集助手',
                model: 'gpt-4',
                status: 'running',
                tools_count: 5
            },
            {
                name: '数据分析师',
                description: '数据分析和可视化专家',
                model: 'claude-3',
                status: 'stopped',
                tools_count: 8
            },
            {
                name: '代码助手',
                description: '编程和代码审查助手',
                model: 'deepseek-coder',
                status: 'running',
                tools_count: 12
            }
        ]
    };

    // 拦截fetch请求
    const originalFetch = window.fetch;
    window.fetch = function(url, options) {
        if (mockAPI[url]) {
            return Promise.resolve({
                ok: true,
                json: () => Promise.resolve(mockAPI[url])
            });
        }
        return originalFetch.apply(this, arguments);
    };
}
