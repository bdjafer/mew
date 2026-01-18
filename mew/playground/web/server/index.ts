import express from 'express';
import { streamText, type CoreMessage } from 'ai';
import { createOpenAI } from '@ai-sdk/openai';
import { createAnthropic } from '@ai-sdk/anthropic';
import { createGoogleGenerativeAI } from '@ai-sdk/google';
import { specTools } from './tools';

const app = express();
app.use(express.json({ limit: '1mb' }));

// CORS for development
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Content-Type');
  if (req.method === 'OPTIONS') {
    return res.sendStatus(200);
  }
  next();
});

type ProviderType = 'openai' | 'anthropic' | 'google';

function createModel(provider: ProviderType, model: string, apiKey: string) {
  switch (provider) {
    case 'openai': {
      const openai = createOpenAI({ apiKey });
      return openai(model || 'gpt-4.1');
    }
    case 'anthropic': {
      const anthropic = createAnthropic({ apiKey });
      return anthropic(model || 'claude-sonnet-4-5-20250929');
    }
    case 'google': {
      const google = createGoogleGenerativeAI({ apiKey });
      return google(model || 'gemini-3.0-flash');
    }
    default:
      throw new Error(`Unknown provider: ${provider}`);
  }
}

interface ChatRequest {
  messages: Array<{ role: 'user' | 'assistant'; content: string }>;
  provider: ProviderType;
  model: string;
  apiKey: string;
  systemPrompt: string;
}

app.post('/api/chat', async (req, res) => {
  try {
    const { messages, provider, model, apiKey, systemPrompt } = req.body as ChatRequest;

    if (!apiKey) {
      return res.status(400).json({ error: 'API key is required' });
    }

    if (!messages || messages.length === 0) {
      return res.status(400).json({ error: 'Messages are required' });
    }

    const modelInstance = createModel(provider, model, apiKey);

    // Convert messages to CoreMessage format
    const coreMessages: CoreMessage[] = messages.map((m) => ({
      role: m.role,
      content: m.content,
    }));

    console.log(`Chat request: provider=${provider}, model=${model}`);

    const result = streamText({
      model: modelInstance,
      system: systemPrompt,
      messages: coreMessages,
      tools: specTools,
      maxSteps: 5,
      onStepFinish: ({ toolCalls, toolResults, finishReason, text }) => {
        console.log(`Step finished: reason=${finishReason}, text=${text?.slice(0, 100)}...`);
        if (toolCalls && toolCalls.length > 0) {
          console.log(
            'Tool calls:',
            toolCalls.map((tc) => `${tc.toolName}(${JSON.stringify(tc.args)})`)
          );
        }
        if (toolResults && toolResults.length > 0) {
          console.log('Tool results:', toolResults.map((tr) => `${tr.toolName}: ${JSON.stringify(tr.result).slice(0, 200)}...`));
        }
      },
      onFinish: ({ finishReason, text, usage }) => {
        console.log(`Stream finished: reason=${finishReason}, tokens=${usage?.totalTokens}`);
      },
    });

    // Stream the response using the AI SDK's data stream format
    result.pipeDataStreamToResponse(res);
  } catch (error) {
    console.error('Chat error:', error);
    res.status(500).json({
      error: error instanceof Error ? error.message : 'Unknown error',
    });
  }
});

// Health check
app.get('/api/health', (req, res) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

const PORT = process.env.PORT || 3001;

app.listen(PORT, () => {
  console.log(`API server running on http://localhost:${PORT}`);
  console.log('Endpoints:');
  console.log('  POST /api/chat - Chat with AI');
  console.log('  GET  /api/health - Health check');
});
