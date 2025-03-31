import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom';
import { Toaster } from '@/components/ui/toaster';
import { cn } from '@/lib/utils';
import {
  Home,
  Users,
  Workflow,
  FileText,
  Settings,
  LayoutDashboard,
  ExternalLink,
  Database,
  Cpu
} from 'lucide-react';

// Import pages
import AgentsPage from '@/pages/agents';
import WorkflowsPage from '@/pages/workflows';
import WorkflowEditor from '@/pages/workflow-editor';
import KnowledgeBasePage from '@/pages/knowledge-base';
import ModelsPage from '@/pages/models';

// Import a simple agent chat for testing
import { AgentChat } from '@/domains/agents/agent/agent-chat';

export default function App() {
  const navigation = [
    { name: 'Dashboard', href: '/', icon: LayoutDashboard },
    { name: 'Agents', href: '/agents', icon: Users },
    { name: 'Workflows', href: '/workflows', icon: Workflow },
    { name: 'Models', href: '/models', icon: Cpu },
    { name: 'Knowledge Base', href: '/knowledge-base', icon: Database }, 
    { name: 'Documentation', href: '/docs', icon: FileText },
  ];
  
  return (
    <Router>
      <div className="flex h-screen overflow-hidden">
        {/* Sidebar */}
        <div className="flex-shrink-0 w-64 bg-background border-r">
          <div className="h-16 flex items-center px-6 border-b">
            <span className="text-xl font-bold">Lumos AI</span>
          </div>
          
          <nav className="p-4 space-y-1">
            {navigation.map((item) => (
              <Link
                key={item.name}
                to={item.href}
                className={cn(
                  'flex items-center px-3 py-2 text-sm font-medium rounded-md',
                  'hover:bg-muted transition-colors duration-150',
                  'group'
                )}
              >
                <item.icon className="mr-3 h-5 w-5" />
                {item.name}
              </Link>
            ))}
          </nav>
          
          <div className="px-4 mt-auto">
            <a
              href="https://github.com/lomusai/lumosai"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center px-3 py-2 text-sm font-medium rounded-md"
            >
              <ExternalLink className="mr-3 h-5 w-5" />
              GitHub
            </a>
            
            <Link
              to="/settings"
              className="flex items-center px-3 py-2 text-sm font-medium rounded-md mt-2"
            >
              <Settings className="mr-3 h-5 w-5" />
              Settings
            </Link>
          </div>
        </div>
        
        {/* Main content */}
        <div className="flex flex-col flex-1 overflow-hidden">
          <main className="flex-1 overflow-y-auto">
            <Routes>
              <Route path="/" element={
                <div className="p-8">
                  <h1 className="text-3xl font-bold mb-6">Welcome to Lumos AI</h1>
                  <p className="text-lg mb-4">
                    Lumos is an open-source framework for building AI applications with agents, workflows, and RAG.
                  </p>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mt-8">
                    <Link to="/agents" className="block p-6 bg-card rounded-lg shadow border hover:shadow-md transition-shadow">
                      <Users className="h-8 w-8 mb-4" />
                      <h2 className="text-xl font-semibold mb-2">Agents</h2>
                      <p>Create and manage AI agents with different capabilities and personalities.</p>
                    </Link>
                    <Link to="/workflows" className="block p-6 bg-card rounded-lg shadow border hover:shadow-md transition-shadow">
                      <Workflow className="h-8 w-8 mb-4" />
                      <h2 className="text-xl font-semibold mb-2">Workflows</h2>
                      <p>Design complex workflows with multiple agents and decision points.</p>
                    </Link>
                    <Link to="/models" className="block p-6 bg-card rounded-lg shadow border hover:shadow-md transition-shadow">
                      <Cpu className="h-8 w-8 mb-4" />
                      <h2 className="text-xl font-semibold mb-2">Models</h2>
                      <p>Configure and test different LLM models with various parameters.</p>
                    </Link>
                    <Link to="/knowledge-base" className="block p-6 bg-card rounded-lg shadow border hover:shadow-md transition-shadow">
                      <Database className="h-8 w-8 mb-4" />
                      <h2 className="text-xl font-semibold mb-2">Knowledge Base</h2>
                      <p>Manage documents and knowledge for your agents to use.</p>
                    </Link>
                  </div>
                </div>
              } />
              <Route path="/agents" element={<AgentsPage />} />
              <Route path="/agents/:id/chat" element={<AgentChatPage />} />
              <Route path="/workflows" element={<WorkflowsPage />} />
              <Route path="/workflows/:id" element={<WorkflowEditor />} />
              <Route path="/workflows/:id/edit" element={<WorkflowEditor />} />
              <Route path="/knowledge-base" element={<KnowledgeBasePage />} />
              <Route path="/models" element={<ModelsPage />} />
              <Route path="/docs" element={<DocsPlaceholder />} />
              <Route path="/settings" element={<SettingsPlaceholder />} />
            </Routes>
          </main>
        </div>
      </div>
      
      <Toaster />
    </Router>
  );
}

// Placeholder components for routes not yet implemented
function AgentChatPage() {
  return (
    <div className="h-screen">
      <AgentChat agentId="test" agentName="Test Agent" />
    </div>
  );
}

function DocsPlaceholder() {
  return (
    <div className="flex items-center justify-center h-[calc(100vh-4rem)]">
      <div className="text-center max-w-md">
        <FileText className="h-12 w-12 mx-auto mb-4" />
        <h2 className="text-2xl font-bold mb-2">Documentation</h2>
        <p className="text-muted-foreground">
          Documentation is coming soon. For now, you can visit the GitHub repository for more information.
        </p>
      </div>
    </div>
  );
}

function SettingsPlaceholder() {
  return (
    <div className="flex items-center justify-center h-[calc(100vh-4rem)]">
      <div className="text-center max-w-md">
        <Settings className="h-12 w-12 mx-auto mb-4" />
        <h2 className="text-2xl font-bold mb-2">Settings</h2>
        <p className="text-muted-foreground">
          Settings panel is coming soon. It will allow you to configure Lumos and manage your preferences.
        </p>
      </div>
    </div>
  );
} 