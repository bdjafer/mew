import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

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
  lastUpdated: string;
}

const SPECS_DIR = path.resolve(__dirname, '../../../../specs');
const OUTPUT_FILE = path.resolve(__dirname, '../server/specs-index.json');

function extractFrontmatter(content: string): Record<string, string> {
  const match = content.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return {};

  const frontmatter: Record<string, string> = {};
  const lines = match[1].split('\n');
  for (const line of lines) {
    const colonIndex = line.indexOf(':');
    if (colonIndex > 0) {
      const key = line.slice(0, colonIndex).trim();
      const value = line.slice(colonIndex + 1).trim().replace(/^["']|["']$/g, '');
      frontmatter[key] = value;
    }
  }
  return frontmatter;
}

function extractHeadings(content: string): string[] {
  const headings: string[] = [];
  const regex = /^#{1,3}\s+(.+)$/gm;
  let match;
  while ((match = regex.exec(content)) !== null) {
    headings.push(match[1].trim());
  }
  return headings;
}

function extractSummary(content: string): string {
  // Remove frontmatter
  const withoutFrontmatter = content.replace(/^---\n[\s\S]*?\n---\n?/, '');

  // Find first paragraph after any heading
  const lines = withoutFrontmatter.split('\n');
  let inParagraph = false;
  let paragraph = '';

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('#')) {
      // Skip headings
      continue;
    }
    if (trimmed === '') {
      if (inParagraph && paragraph.length > 50) {
        break;
      }
      continue;
    }
    if (!trimmed.startsWith('```') && !trimmed.startsWith('|') && !trimmed.startsWith('-')) {
      inParagraph = true;
      paragraph += (paragraph ? ' ' : '') + trimmed;
      if (paragraph.length > 200) {
        break;
      }
    }
  }

  return paragraph.slice(0, 250) + (paragraph.length > 250 ? '...' : '');
}

function extractKeywords(content: string, title: string): string[] {
  const keywords = new Set<string>();

  // Add title words
  title.toLowerCase().split(/\s+/).forEach((w) => {
    if (w.length > 3) keywords.add(w);
  });

  // Extract MEW keywords from content
  const mewKeywords = [
    'match',
    'spawn',
    'kill',
    'link',
    'unlink',
    'set',
    'walk',
    'return',
    'where',
    'node',
    'edge',
    'constraint',
    'required',
    'unique',
    'indexed',
    'optional',
    'union',
    'type',
    'ontology',
    'transaction',
    'begin',
    'commit',
    'rollback',
  ];

  const lowerContent = content.toLowerCase();
  for (const kw of mewKeywords) {
    if (lowerContent.includes(kw)) {
      keywords.add(kw);
    }
  }

  return Array.from(keywords);
}

function scanDirectory(dir: string, category: string = ''): SpecEntry[] {
  const entries: SpecEntry[] = [];

  const items = fs.readdirSync(dir);
  for (const item of items) {
    const fullPath = path.join(dir, item);
    const stat = fs.statSync(fullPath);

    if (stat.isDirectory()) {
      entries.push(...scanDirectory(fullPath, item));
    } else if (item.endsWith('.md') && item !== 'spec_template.md') {
      const content = fs.readFileSync(fullPath, 'utf-8');
      const frontmatter = extractFrontmatter(content);
      const headings = extractHeadings(content);
      const relativePath = path.relative(SPECS_DIR, fullPath);

      // Determine title from frontmatter or first heading or filename
      let title = frontmatter.spec || headings[0] || item.replace('.md', '');
      title = title.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());

      const summary = extractSummary(content);
      const keywords = extractKeywords(content, title);

      entries.push({
        file: relativePath,
        title,
        category: category || 'root',
        summary,
        headings: headings.slice(0, 10),
        keywords,
      });
    }
  }

  return entries;
}

function buildIndex(): SpecsIndex {
  console.log(`Scanning specs directory: ${SPECS_DIR}`);

  const specs = scanDirectory(SPECS_DIR);
  const categories = [...new Set(specs.map((s) => s.category))].sort();

  console.log(`Found ${specs.length} specs in ${categories.length} categories`);

  return {
    specs,
    categories,
    lastUpdated: new Date().toISOString(),
  };
}

// Main
const index = buildIndex();

// Ensure output directory exists
const outputDir = path.dirname(OUTPUT_FILE);
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

fs.writeFileSync(OUTPUT_FILE, JSON.stringify(index, null, 2));
console.log(`Wrote specs index to: ${OUTPUT_FILE}`);
console.log(`Index size: ${(fs.statSync(OUTPUT_FILE).size / 1024).toFixed(1)} KB`);
