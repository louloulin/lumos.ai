import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";
import { ThemeSwitch } from "./components/theme-switch";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container mx-auto p-4 flex flex-col items-center min-h-screen">
      <div className="w-full max-w-4xl flex justify-between items-center mb-8">
        <h1 className="text-3xl font-bold">LumosAI UI</h1>
        <ThemeSwitch />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8 w-full max-w-4xl">
        <div className="p-6 rounded-lg shadow-sm border" style={{ backgroundColor: "rgb(var(--card))" }}>
          <h2 className="text-xl font-semibold mb-4">基础组件</h2>
          <div className="space-y-4">
            <div>
              <h3 className="text-lg font-medium mb-2">按钮</h3>
              <div className="flex flex-wrap gap-2">
                <Button>默认</Button>
                <Button variant="secondary">次要</Button>
                <Button variant="outline">轮廓</Button>
                <Button variant="destructive">警告</Button>
                <Button variant="ghost">幽灵</Button>
                <Button variant="link">链接</Button>
              </div>
            </div>
            
            <div>
              <h3 className="text-lg font-medium mb-2">输入框</h3>
              <Input placeholder="请输入文本..." />
            </div>
          </div>
        </div>
        
        <div className="p-6 rounded-lg shadow-sm border" style={{ backgroundColor: "rgb(var(--card))" }}>
          <h2 className="text-xl font-semibold mb-4">Tauri 集成</h2>
          <form
            className="space-y-4"
            onSubmit={(e) => {
              e.preventDefault();
              greet();
            }}
          >
            <Input
              value={name}
              onChange={(e) => setName(e.currentTarget.value)}
              placeholder="输入名称..."
            />
            <Button type="submit">问候</Button>
            {greetMsg && <p className="p-4 rounded-md" style={{ backgroundColor: "rgb(var(--muted))" }}>{greetMsg}</p>}
          </form>
        </div>
      </div>
      
      <footer className="mt-auto pt-8 text-center text-sm" style={{ color: "rgb(var(--muted-foreground))" }}>
        LumosAI UI Framework - 基于 Tauri、React 和 Tailwind CSS
      </footer>
    </main>
  );
}

export default App;
