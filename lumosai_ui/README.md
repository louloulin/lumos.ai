# Lumos UI

Lumos UI is the frontend component of the Lumos AI platform, providing an intuitive interface for building, testing, and deploying AI agents, workflows, and knowledge bases.

## Features

- **Agent Management**: Create, edit, and test AI agents
- **Workflow Editor**: Visual editor for building and configuring workflows
- **Model Management**: Configure and test different LLM models
- **Knowledge Base**: Upload, index, and search through documents
- **Deployment Management**: Deploy and monitor your AI applications

## Getting Started

### Prerequisites

- Node.js 18+ and pnpm
- Lumos backend services running locally or remotely

### Installation

1. Clone the repository
   ```bash
   git clone https://github.com/lomusai/lumosai.git
   cd lumosai/lumos_ui
   ```

2. Install dependencies
   ```bash
   pnpm install
   ```

3. Create a `.env` file with your configuration
   ```bash
   VITE_API_BASE_URL=http://localhost:3000
   ```

4. Start the development server
   ```bash
   pnpm dev
   ```

## Development

### Project Structure

```
lumos_ui/
├── src/               # Source code
│   ├── components/    # Reusable UI components
│   ├── domains/       # Domain-specific components and logic
│   ├── hooks/         # Custom React hooks
│   ├── lib/           # Utilities and helpers
│   ├── pages/         # Top-level page components
│   ├── services/      # API and service integrations
│   └── types.ts       # TypeScript type definitions
├── public/            # Static assets
└── ...                # Configuration files
```

### Commands

- `pnpm dev` - Start the development server
- `pnpm build` - Build for production
- `pnpm preview` - Preview the production build locally

## Integration with Lumos CLI

Lumos UI is designed to work seamlessly with the Lumos CLI development environment. When running `lumos dev`, the CLI will automatically:

1. Start the backend API server
2. Launch the UI development server
3. Set up file watching for hot reloading
4. Provide unified logging

## Contributing

1. Create a feature branch
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes and commit
   ```bash
   git commit -m "Add your feature description"
   ```

3. Push to your branch
   ```bash
   git push origin feature/your-feature-name
   ```

4. Create a pull request

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for the detailed implementation plan.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 