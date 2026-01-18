import { tool } from 'ai';
import { z } from 'zod';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load specs index
const SPECS_INDEX_PATH = path.resolve(__dirname, 'specs-index.json');
const SPECS_DIR = path.resolve(__dirname, '../../../specs');

interface SpecEntry {
  file: string;
  title: string;
  category: string;
  summary: string;
  headings: string[];
  keywords: string[];
}

interface SpecsIndex {
  specs: SpecEntry[];
  categories: string[];
}

function loadSpecsIndex(): SpecsIndex {
  if (!fs.existsSync(SPECS_INDEX_PATH)) {
    console.warn('Specs index not found. Run: npm run build:specs-index');
    return { specs: [], categories: [] };
  }
  return JSON.parse(fs.readFileSync(SPECS_INDEX_PATH, 'utf-8'));
}

function searchSpecs(query: string, category?: string): SpecEntry[] {
  const index = loadSpecsIndex();
  const queryLower = query.toLowerCase();
  const queryWords = queryLower.split(/\s+/).filter((w) => w.length > 2);

  let specs = index.specs;

  // Filter by category if specified
  if (category && category !== 'all') {
    specs = specs.filter((s) => s.category === category);
  }

  // Score each spec
  const scored = specs.map((spec) => {
    let score = 0;

    // Title match (highest weight)
    const titleLower = spec.title.toLowerCase();
    if (titleLower.includes(queryLower)) {
      score += 100;
    }
    for (const word of queryWords) {
      if (titleLower.includes(word)) score += 20;
    }

    // Keyword match
    for (const kw of spec.keywords) {
      if (queryLower.includes(kw)) score += 15;
      for (const word of queryWords) {
        if (kw.includes(word)) score += 10;
      }
    }

    // Heading match
    for (const heading of spec.headings) {
      const headingLower = heading.toLowerCase();
      if (headingLower.includes(queryLower)) score += 30;
      for (const word of queryWords) {
        if (headingLower.includes(word)) score += 5;
      }
    }

    // Summary match
    const summaryLower = spec.summary.toLowerCase();
    for (const word of queryWords) {
      if (summaryLower.includes(word)) score += 3;
    }

    return { spec, score };
  });

  // Sort by score and return top results
  return scored
    .filter((s) => s.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 8)
    .map((s) => s.spec);
}

function readSpecFile(file: string, section?: string): string {
  const fullPath = path.join(SPECS_DIR, file);

  if (!fs.existsSync(fullPath)) {
    return `Error: Spec file not found: ${file}`;
  }

  let content = fs.readFileSync(fullPath, 'utf-8');

  // If section specified, extract just that section
  if (section) {
    const sectionRegex = new RegExp(
      `^(#{1,3})\\s+${section.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s*$`,
      'im'
    );
    const match = content.match(sectionRegex);

    if (match) {
      const startIndex = match.index!;
      const headingLevel = match[1].length;

      // Find the next heading at same or higher level
      const remainingContent = content.slice(startIndex + match[0].length);
      const nextHeadingRegex = new RegExp(`^#{1,${headingLevel}}\\s+`, 'm');
      const nextMatch = remainingContent.match(nextHeadingRegex);

      if (nextMatch) {
        content = content.slice(startIndex, startIndex + match[0].length + nextMatch.index!);
      } else {
        content = content.slice(startIndex);
      }
    }
  }

  // Truncate if too long
  const MAX_LENGTH = 12000;
  if (content.length > MAX_LENGTH) {
    content = content.slice(0, MAX_LENGTH) + '\n\n... [Content truncated]';
  }

  return content;
}

export const specTools = {
  search_specs: tool({
    description:
      'Search MEW specifications by keyword or topic. Returns matching spec files with summaries. Use this to find relevant documentation.',
    parameters: z.object({
      query: z
        .string()
        .describe(
          'Search query (e.g., "MATCH statement", "edge constraints", "transactions", "required modifier")'
        ),
      category: z
        .enum([
          'all',
          'core',
          'declarations',
          'expressions',
          'statements',
          'modifiers',
          'types',
          'patterns',
          'literals',
        ])
        .optional()
        .describe('Optional category to filter results'),
    }),
    execute: async ({ query, category }) => {
      const results = searchSpecs(query, category);

      if (results.length === 0) {
        return { found: false, message: `No specs found matching "${query}"`, results: [] };
      }

      return {
        found: true,
        count: results.length,
        results: results.map((r) => ({
          file: r.file,
          title: r.title,
          category: r.category,
          summary: r.summary,
        })),
      };
    },
  }),

  read_spec: tool({
    description:
      'Read the full content of a specific MEW specification file. Use this after search_specs to get detailed information.',
    parameters: z.object({
      file: z
        .string()
        .describe(
          'Spec file path from search results (e.g., "statements/match.md", "modifiers/unique.md")'
        ),
      section: z
        .string()
        .optional()
        .describe('Optional section heading to read only that section (e.g., "Examples", "Syntax")'),
    }),
    execute: async ({ file, section }) => {
      const content = readSpecFile(file, section);
      const isError = content.startsWith('Error:');

      return {
        success: !isError,
        file,
        section: section || null,
        content,
      };
    },
  }),

  list_specs: tool({
    description: 'List all available MEW specification files organized by category.',
    parameters: z.object({
      category: z
        .enum([
          'all',
          'core',
          'declarations',
          'expressions',
          'statements',
          'modifiers',
          'types',
          'patterns',
          'literals',
          'root',
        ])
        .optional()
        .describe('Category to list, or "all" for everything'),
    }),
    execute: async ({ category }) => {
      const index = loadSpecsIndex();
      let specs = index.specs;

      if (category && category !== 'all') {
        specs = specs.filter((s) => s.category === category);
      }

      // Group by category
      const grouped: Record<string, { file: string; title: string }[]> = {};
      for (const spec of specs) {
        if (!grouped[spec.category]) {
          grouped[spec.category] = [];
        }
        grouped[spec.category].push({ file: spec.file, title: spec.title });
      }

      return {
        total: specs.length,
        categories: Object.keys(grouped).sort(),
        specs: grouped,
      };
    },
  }),

  // Editor tools - these return actions that the client will process
  edit_ontology: tool({
    description:
      'Update the ontology in the editor. Use this to set or modify the ontology schema. The content should be valid MEW ontology code.',
    parameters: z.object({
      content: z.string().describe('The complete MEW ontology code to set in the editor'),
      explanation: z.string().optional().describe('Brief explanation of what the ontology does'),
    }),
    execute: async ({ content, explanation }) => {
      return {
        action: 'edit_ontology',
        content,
        explanation,
        success: true,
      };
    },
  }),

  edit_query: tool({
    description:
      'Update the query in the editor. Use this to set or modify the query. The content should be valid MEW query code (MATCH, SPAWN, etc.).',
    parameters: z.object({
      content: z.string().describe('The complete MEW query code to set in the editor'),
      explanation: z.string().optional().describe('Brief explanation of what the query does'),
    }),
    execute: async ({ content, explanation }) => {
      return {
        action: 'edit_query',
        content,
        explanation,
        success: true,
      };
    },
  }),

  execute_query: tool({
    description:
      'Execute the current query in the editor. The query will be run against the current ontology and results will be displayed in the visualization.',
    parameters: z.object({
      waitForResults: z.boolean().optional().describe('Whether to wait and report results (default: false)'),
    }),
    execute: async ({ waitForResults }) => {
      return {
        action: 'execute_query',
        waitForResults: waitForResults || false,
        success: true,
      };
    },
  }),

  generate_seed: tool({
    description:
      'Generate and execute seed data for the current ontology. Use SPAWN statements to create nodes and LINK statements to create edges. This will populate the graph with sample data.',
    parameters: z.object({
      content: z.string().describe('MEW mutation statements (SPAWN, LINK) to create seed data'),
      explanation: z.string().optional().describe('Brief explanation of the seed data'),
    }),
    execute: async ({ content, explanation }) => {
      return {
        action: 'generate_seed',
        content,
        explanation,
        success: true,
      };
    },
  }),
};
