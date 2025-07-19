// 工作流可视化编辑器
class WorkflowEditor {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.canvas = null;
        this.nodes = new Map();
        this.connections = new Map();
        this.selectedNode = null;
        this.draggedNode = null;
        this.isConnecting = false;
        this.connectionStart = null;
        
        this.nodeTypes = {
            agent: { color: '#3B82F6', icon: 'fas fa-user-robot' },
            tool: { color: '#10B981', icon: 'fas fa-tools' },
            condition: { color: '#F59E0B', icon: 'fas fa-code-branch' },
            parallel: { color: '#8B5CF6', icon: 'fas fa-layer-group' },
            start: { color: '#6B7280', icon: 'fas fa-play' },
            end: { color: '#EF4444', icon: 'fas fa-stop' }
        };
        
        this.init();
    }

    init() {
        this.createCanvas();
        this.setupEventListeners();
        this.createToolbar();
        this.addDefaultNodes();
    }

    createCanvas() {
        this.container.innerHTML = `
            <div class="workflow-editor-container relative w-full h-96 bg-gray-50 border-2 border-dashed border-gray-300 rounded-lg overflow-hidden">
                <div id="workflow-toolbar" class="absolute top-4 left-4 z-10 flex space-x-2"></div>
                <svg id="workflow-canvas" class="w-full h-full cursor-crosshair">
                    <defs>
                        <marker id="arrowhead" markerWidth="10" markerHeight="7" 
                                refX="9" refY="3.5" orient="auto">
                            <polygon points="0 0, 10 3.5, 0 7" fill="#6B7280" />
                        </marker>
                    </defs>
                </svg>
                <div id="workflow-nodes" class="absolute inset-0 pointer-events-none"></div>
            </div>
        `;
        
        this.canvas = document.getElementById('workflow-canvas');
        this.nodesContainer = document.getElementById('workflow-nodes');
    }

    createToolbar() {
        const toolbar = document.getElementById('workflow-toolbar');
        
        const tools = [
            { type: 'agent', label: 'Agent', icon: 'fas fa-user-robot' },
            { type: 'tool', label: '工具', icon: 'fas fa-tools' },
            { type: 'condition', label: '条件', icon: 'fas fa-code-branch' },
            { type: 'parallel', label: '并行', icon: 'fas fa-layer-group' }
        ];
        
        tools.forEach(tool => {
            const button = document.createElement('button');
            button.className = 'bg-white border border-gray-300 rounded px-3 py-2 text-sm hover:bg-gray-50 flex items-center space-x-2';
            button.innerHTML = `<i class="${tool.icon}"></i><span>${tool.label}</span>`;
            button.addEventListener('click', () => this.addNode(tool.type));
            toolbar.appendChild(button);
        });

        // 添加控制按钮
        const controls = document.createElement('div');
        controls.className = 'flex space-x-2 ml-4';
        controls.innerHTML = `
            <button id="save-workflow" class="bg-blue-500 text-white px-3 py-2 rounded text-sm hover:bg-blue-600">
                <i class="fas fa-save mr-1"></i>保存
            </button>
            <button id="run-workflow" class="bg-green-500 text-white px-3 py-2 rounded text-sm hover:bg-green-600">
                <i class="fas fa-play mr-1"></i>运行
            </button>
            <button id="clear-workflow" class="bg-red-500 text-white px-3 py-2 rounded text-sm hover:bg-red-600">
                <i class="fas fa-trash mr-1"></i>清空
            </button>
        `;
        toolbar.appendChild(controls);

        // 绑定控制按钮事件
        document.getElementById('save-workflow').addEventListener('click', () => this.saveWorkflow());
        document.getElementById('run-workflow').addEventListener('click', () => this.runWorkflow());
        document.getElementById('clear-workflow').addEventListener('click', () => this.clearWorkflow());
    }

    setupEventListeners() {
        // 画布点击事件
        this.canvas.addEventListener('click', (e) => {
            if (this.isConnecting) {
                this.handleConnectionClick(e);
            } else {
                this.deselectAll();
            }
        });

        // 键盘事件
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Delete' && this.selectedNode) {
                this.deleteNode(this.selectedNode);
            }
            if (e.key === 'Escape') {
                this.cancelConnection();
            }
        });
    }

    addDefaultNodes() {
        // 添加开始和结束节点
        this.createNode('start', 100, 200, '开始');
        this.createNode('end', 700, 200, '结束');
    }

    addNode(type, x = null, y = null) {
        const rect = this.container.getBoundingClientRect();
        const nodeX = x || rect.width / 2 + Math.random() * 200 - 100;
        const nodeY = y || rect.height / 2 + Math.random() * 100 - 50;
        
        const nodeId = this.generateNodeId();
        const label = this.getDefaultLabel(type);
        
        this.createNode(type, nodeX, nodeY, label, nodeId);
    }

    createNode(type, x, y, label, id = null) {
        const nodeId = id || this.generateNodeId();
        const nodeConfig = this.nodeTypes[type];
        
        const nodeElement = document.createElement('div');
        nodeElement.className = 'workflow-node absolute bg-white border-2 rounded-lg p-3 shadow-lg cursor-move pointer-events-auto';
        nodeElement.style.left = `${x}px`;
        nodeElement.style.top = `${y}px`;
        nodeElement.style.borderColor = nodeConfig.color;
        nodeElement.dataset.nodeId = nodeId;
        nodeElement.dataset.nodeType = type;
        
        nodeElement.innerHTML = `
            <div class="flex items-center space-x-2 mb-2">
                <div class="w-8 h-8 rounded-full flex items-center justify-center text-white" style="background-color: ${nodeConfig.color}">
                    <i class="${nodeConfig.icon} text-sm"></i>
                </div>
                <div class="flex-1">
                    <div class="font-medium text-sm text-gray-800">${label}</div>
                    <div class="text-xs text-gray-500">${type}</div>
                </div>
            </div>
            <div class="flex justify-between items-center">
                <button class="connect-btn text-xs bg-gray-100 hover:bg-gray-200 px-2 py-1 rounded">
                    <i class="fas fa-link"></i>
                </button>
                <button class="config-btn text-xs bg-blue-100 hover:bg-blue-200 px-2 py-1 rounded">
                    <i class="fas fa-cog"></i>
                </button>
            </div>
        `;
        
        this.nodesContainer.appendChild(nodeElement);
        
        // 存储节点数据
        this.nodes.set(nodeId, {
            id: nodeId,
            type: type,
            label: label,
            x: x,
            y: y,
            element: nodeElement,
            config: {}
        });
        
        this.setupNodeEvents(nodeElement);
    }

    setupNodeEvents(nodeElement) {
        const nodeId = nodeElement.dataset.nodeId;
        
        // 节点选择
        nodeElement.addEventListener('click', (e) => {
            e.stopPropagation();
            this.selectNode(nodeId);
        });
        
        // 拖拽
        nodeElement.addEventListener('mousedown', (e) => {
            if (e.target.closest('.connect-btn') || e.target.closest('.config-btn')) {
                return;
            }
            this.startDrag(nodeId, e);
        });
        
        // 连接按钮
        const connectBtn = nodeElement.querySelector('.connect-btn');
        connectBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            this.startConnection(nodeId);
        });
        
        // 配置按钮
        const configBtn = nodeElement.querySelector('.config-btn');
        configBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            this.configureNode(nodeId);
        });
    }

    selectNode(nodeId) {
        this.deselectAll();
        this.selectedNode = nodeId;
        const node = this.nodes.get(nodeId);
        if (node) {
            node.element.classList.add('ring-2', 'ring-blue-500');
        }
    }

    deselectAll() {
        this.selectedNode = null;
        this.nodes.forEach(node => {
            node.element.classList.remove('ring-2', 'ring-blue-500');
        });
    }

    startDrag(nodeId, e) {
        this.draggedNode = nodeId;
        const node = this.nodes.get(nodeId);
        const rect = this.container.getBoundingClientRect();
        
        const offsetX = e.clientX - rect.left - node.x;
        const offsetY = e.clientY - rect.top - node.y;
        
        const handleMouseMove = (e) => {
            const newX = e.clientX - rect.left - offsetX;
            const newY = e.clientY - rect.top - offsetY;
            
            this.moveNode(nodeId, newX, newY);
        };
        
        const handleMouseUp = () => {
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', handleMouseUp);
            this.draggedNode = null;
        };
        
        document.addEventListener('mousemove', handleMouseMove);
        document.addEventListener('mouseup', handleMouseUp);
    }

    moveNode(nodeId, x, y) {
        const node = this.nodes.get(nodeId);
        if (node) {
            node.x = x;
            node.y = y;
            node.element.style.left = `${x}px`;
            node.element.style.top = `${y}px`;
            
            // 更新连接线
            this.updateConnections(nodeId);
        }
    }

    startConnection(nodeId) {
        this.isConnecting = true;
        this.connectionStart = nodeId;
        this.canvas.style.cursor = 'crosshair';
        
        // 高亮可连接的节点
        this.nodes.forEach((node, id) => {
            if (id !== nodeId) {
                node.element.classList.add('ring-2', 'ring-green-300');
            }
        });
    }

    handleConnectionClick(e) {
        const target = e.target.closest('.workflow-node');
        if (target) {
            const targetId = target.dataset.nodeId;
            if (targetId !== this.connectionStart) {
                this.createConnection(this.connectionStart, targetId);
            }
        }
        this.cancelConnection();
    }

    cancelConnection() {
        this.isConnecting = false;
        this.connectionStart = null;
        this.canvas.style.cursor = 'default';
        
        // 移除高亮
        this.nodes.forEach(node => {
            node.element.classList.remove('ring-2', 'ring-green-300');
        });
    }

    createConnection(fromId, toId) {
        const connectionId = `${fromId}-${toId}`;
        if (this.connections.has(connectionId)) {
            return; // 连接已存在
        }
        
        const fromNode = this.nodes.get(fromId);
        const toNode = this.nodes.get(toId);
        
        if (fromNode && toNode) {
            const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
            line.setAttribute('stroke', '#6B7280');
            line.setAttribute('stroke-width', '2');
            line.setAttribute('marker-end', 'url(#arrowhead)');
            line.dataset.connectionId = connectionId;
            
            this.canvas.appendChild(line);
            
            this.connections.set(connectionId, {
                id: connectionId,
                from: fromId,
                to: toId,
                element: line
            });
            
            this.updateConnectionLine(connectionId);
        }
    }

    updateConnections(nodeId) {
        this.connections.forEach((connection, id) => {
            if (connection.from === nodeId || connection.to === nodeId) {
                this.updateConnectionLine(id);
            }
        });
    }

    updateConnectionLine(connectionId) {
        const connection = this.connections.get(connectionId);
        if (!connection) return;
        
        const fromNode = this.nodes.get(connection.from);
        const toNode = this.nodes.get(connection.to);
        
        if (fromNode && toNode) {
            const fromX = fromNode.x + 50; // 节点中心
            const fromY = fromNode.y + 30;
            const toX = toNode.x + 50;
            const toY = toNode.y + 30;
            
            connection.element.setAttribute('x1', fromX);
            connection.element.setAttribute('y1', fromY);
            connection.element.setAttribute('x2', toX);
            connection.element.setAttribute('y2', toY);
        }
    }

    configureNode(nodeId) {
        const node = this.nodes.get(nodeId);
        if (!node) return;
        
        // 简单的配置对话框
        const newLabel = prompt(`配置${node.type}节点:`, node.label);
        if (newLabel && newLabel !== node.label) {
            node.label = newLabel;
            const labelElement = node.element.querySelector('.font-medium');
            labelElement.textContent = newLabel;
        }
    }

    deleteNode(nodeId) {
        const node = this.nodes.get(nodeId);
        if (!node) return;
        
        // 删除相关连接
        const connectionsToDelete = [];
        this.connections.forEach((connection, id) => {
            if (connection.from === nodeId || connection.to === nodeId) {
                connectionsToDelete.push(id);
            }
        });
        
        connectionsToDelete.forEach(id => {
            const connection = this.connections.get(id);
            connection.element.remove();
            this.connections.delete(id);
        });
        
        // 删除节点
        node.element.remove();
        this.nodes.delete(nodeId);
        this.selectedNode = null;
    }

    clearWorkflow() {
        if (confirm('确定要清空工作流吗？')) {
            this.nodes.clear();
            this.connections.clear();
            this.nodesContainer.innerHTML = '';
            this.canvas.innerHTML = this.canvas.querySelector('defs').outerHTML;
            this.addDefaultNodes();
        }
    }

    saveWorkflow() {
        const workflowData = {
            nodes: Array.from(this.nodes.values()).map(node => ({
                id: node.id,
                type: node.type,
                label: node.label,
                x: node.x,
                y: node.y,
                config: node.config
            })),
            connections: Array.from(this.connections.values()).map(conn => ({
                from: conn.from,
                to: conn.to
            }))
        };
        
        console.log('保存工作流:', workflowData);
        alert('工作流已保存到控制台');
    }

    runWorkflow() {
        const workflowData = this.getWorkflowData();
        console.log('运行工作流:', workflowData);
        alert('工作流开始执行');
    }

    getWorkflowData() {
        return {
            nodes: Array.from(this.nodes.values()),
            connections: Array.from(this.connections.values())
        };
    }

    generateNodeId() {
        return 'node_' + Math.random().toString(36).substr(2, 9);
    }

    getDefaultLabel(type) {
        const labels = {
            agent: 'AI Agent',
            tool: '工具',
            condition: '条件判断',
            parallel: '并行处理',
            start: '开始',
            end: '结束'
        };
        return labels[type] || type;
    }
}

// 在工作流页面加载时初始化编辑器
document.addEventListener('DOMContentLoaded', () => {
    // 等待页面切换到工作流页面时再初始化
    const originalNavigateTo = window.lumosAI?.navigateTo;
    if (originalNavigateTo) {
        window.lumosAI.navigateTo = function(page) {
            originalNavigateTo.call(this, page);
            if (page === 'workflows' && !window.workflowEditor) {
                setTimeout(() => {
                    window.workflowEditor = new WorkflowEditor('workflow-canvas');
                }, 100);
            }
        };
    }
});
