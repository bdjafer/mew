import type { GraphData, NodeData, Layout } from '../types';

const COLORS = {
  bgPrimary: '#0f0f0f',
  nodeBg: '#2a2a2a',
  nodeBorder: '#444',
  nodeFocal: '#6366f1',
  nodeSelected: '#22c55e',
  nodePeripheral: '#1f1f1f',
  textPrimary: '#e5e5e5',
  textSecondary: '#a0a0a0',
  textPeripheral: '#666',
  edge: '#555',
  edgeFocal: '#6366f1',
};

export class Renderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private graph: GraphData | null = null;
  private layout: Layout | null = null;
  private focal: Set<number> = new Set();
  private selected: number | null = null;
  private highlightedNode: number | null = null;
  private viewport = { x: 0, y: 0 };
  private zoom = 1;
  private isDragging = false;
  private lastMouse = { x: 0, y: 0 };
  onNodeClick: ((node: NodeData) => void) | null = null;
  onNodeHover: ((node: NodeData | null) => void) | null = null;
  onZoomChange: ((zoom: number) => void) | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.resize();
    this.setupEventListeners();
  }

  resize() {
    const rect = this.canvas.parentElement!.getBoundingClientRect();
    this.canvas.width = rect.width;
    this.canvas.height = rect.height;
    this.draw();
  }

  private setupEventListeners() {
    this.canvas.addEventListener('mousedown', this.handleMouseDown.bind(this));
    this.canvas.addEventListener('mousemove', this.handleMouseMove.bind(this));
    this.canvas.addEventListener('mouseup', this.handleMouseUp.bind(this));
    this.canvas.addEventListener('wheel', this.handleWheel.bind(this));
    this.canvas.addEventListener('click', this.handleClick.bind(this));
  }

  private handleMouseDown(e: MouseEvent) {
    this.isDragging = true;
    this.lastMouse = { x: e.clientX, y: e.clientY };
    this.canvas.style.cursor = 'grabbing';
  }

  private handleMouseMove(e: MouseEvent) {
    if (this.isDragging) {
      const dx = e.clientX - this.lastMouse.x;
      const dy = e.clientY - this.lastMouse.y;
      this.viewport.x += dx / this.zoom;
      this.viewport.y += dy / this.zoom;
      this.lastMouse = { x: e.clientX, y: e.clientY };
      this.draw();
    } else {
      const pos = this.screenToWorld(e.clientX, e.clientY);
      const node = this.findNodeAt(pos.x, pos.y);
      if (node) {
        this.canvas.style.cursor = 'pointer';
        this.onNodeHover?.(node);
      } else {
        this.canvas.style.cursor = 'grab';
        this.onNodeHover?.(null);
      }
    }
  }

  private handleMouseUp() {
    this.isDragging = false;
    this.canvas.style.cursor = 'grab';
  }

  private handleWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.9 : 1.1;
    this.setZoom(this.zoom * factor);
  }

  private setZoom(newZoom: number) {
    this.zoom = Math.max(0.1, Math.min(5, newZoom));
    this.onZoomChange?.(this.zoom);
    this.draw();
  }

  zoomIn() {
    this.setZoom(this.zoom * 1.2);
  }

  zoomOut() {
    this.setZoom(this.zoom / 1.2);
  }

  fitToView() {
    if (!this.graph || !this.layout || this.layout.nodes.size === 0) return;
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const nl of this.layout.nodes.values()) {
      minX = Math.min(minX, nl.x);
      minY = Math.min(minY, nl.y);
      maxX = Math.max(maxX, nl.x + nl.width);
      maxY = Math.max(maxY, nl.y + nl.height);
    }
    const graphWidth = maxX - minX;
    const graphHeight = maxY - minY;
    const padding = 50;
    const scaleX = (this.canvas.width - padding * 2) / graphWidth;
    const scaleY = (this.canvas.height - padding * 2) / graphHeight;
    this.zoom = Math.min(scaleX, scaleY, 2);
    this.viewport.x = -minX + padding / this.zoom + (this.canvas.width / this.zoom - graphWidth) / 2;
    this.viewport.y = -minY + padding / this.zoom + (this.canvas.height / this.zoom - graphHeight) / 2;
    this.onZoomChange?.(this.zoom);
    this.draw();
  }

  private handleClick(e: MouseEvent) {
    if (this.isDragging) return;
    const pos = this.screenToWorld(e.clientX, e.clientY);
    const node = this.findNodeAt(pos.x, pos.y);
    if (node) {
      this.onNodeClick?.(node);
    }
  }

  private screenToWorld(screenX: number, screenY: number): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    return {
      x: (screenX - rect.left) / this.zoom - this.viewport.x,
      y: (screenY - rect.top) / this.zoom - this.viewport.y,
    };
  }

  worldToScreen(worldX: number, worldY: number): { x: number; y: number } {
    return {
      x: (worldX + this.viewport.x) * this.zoom,
      y: (worldY + this.viewport.y) * this.zoom,
    };
  }

  private findNodeAt(x: number, y: number): NodeData | null {
    if (!this.graph || !this.layout) return null;
    for (const node of this.graph.nodes) {
      const nl = this.layout.nodes.get(node.id);
      if (nl && x >= nl.x && x <= nl.x + nl.width && y >= nl.y && y <= nl.y + nl.height) {
        return node;
      }
    }
    return null;
  }

  render(graph: GraphData, layout: Layout, focal: Set<number>, selected: number | null) {
    this.graph = graph;
    this.layout = layout;
    this.focal = focal;
    this.selected = selected;
    this.draw();
  }

  highlightNode(id: number) {
    this.highlightedNode = id;
    this.draw();
  }

  clearHighlight() {
    this.highlightedNode = null;
    this.draw();
  }

  private draw() {
    if (!this.graph || !this.layout) {
      this.ctx.fillStyle = COLORS.bgPrimary;
      this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
      return;
    }
    this.ctx.fillStyle = COLORS.bgPrimary;
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    this.ctx.save();
    this.ctx.scale(this.zoom, this.zoom);
    this.ctx.translate(this.viewport.x, this.viewport.y);
    this.drawEdges(false);
    this.drawEdges(true);
    this.drawNodes(false);
    this.drawNodes(true);
    this.ctx.restore();
  }

  private drawEdges(focalOnly: boolean) {
    if (!this.graph || !this.layout) return;
    for (const edge of this.graph.edges) {
      const el = this.layout.edges.get(edge.id);
      if (!el) continue;
      const isFocal = edge.targets.some(t => this.focal.has(t));
      if (focalOnly !== isFocal) continue;
      this.ctx.beginPath();
      this.ctx.strokeStyle = isFocal ? COLORS.edgeFocal : COLORS.edge;
      this.ctx.lineWidth = isFocal ? 2 : 1;
      this.ctx.globalAlpha = isFocal ? 1 : 0.4;
      if (el.path.length >= 2) {
        this.ctx.moveTo(el.path[0][0], el.path[0][1]);
        for (let i = 1; i < el.path.length; i++) {
          this.ctx.lineTo(el.path[i][0], el.path[i][1]);
        }
      }
      this.ctx.stroke();
      this.ctx.globalAlpha = 1;
      if (el.path.length >= 2) {
        this.drawArrow(el.path[el.path.length - 2], el.path[el.path.length - 1], isFocal);
      }
    }
  }

  private drawArrow(from: [number, number], to: [number, number], isFocal: boolean) {
    const headLen = 10;
    const dx = to[0] - from[0];
    const dy = to[1] - from[1];
    const angle = Math.atan2(dy, dx);
    this.ctx.beginPath();
    this.ctx.moveTo(to[0], to[1]);
    this.ctx.lineTo(to[0] - headLen * Math.cos(angle - Math.PI / 6), to[1] - headLen * Math.sin(angle - Math.PI / 6));
    this.ctx.lineTo(to[0] - headLen * Math.cos(angle + Math.PI / 6), to[1] - headLen * Math.sin(angle + Math.PI / 6));
    this.ctx.closePath();
    this.ctx.fillStyle = isFocal ? COLORS.edgeFocal : COLORS.edge;
    this.ctx.globalAlpha = isFocal ? 1 : 0.4;
    this.ctx.fill();
    this.ctx.globalAlpha = 1;
  }

  private drawNodes(focalOnly: boolean) {
    if (!this.graph || !this.layout) return;
    for (const node of this.graph.nodes) {
      const nl = this.layout.nodes.get(node.id);
      if (!nl) continue;
      const isFocal = this.focal.size === 0 || this.focal.has(node.id);
      if (focalOnly !== isFocal) continue;
      const isSelected = this.selected === node.id;
      const isHighlighted = this.highlightedNode === node.id;
      this.ctx.globalAlpha = isFocal ? 1 : 0.4;
      this.ctx.fillStyle = isFocal ? COLORS.nodeBg : COLORS.nodePeripheral;
      this.ctx.strokeStyle = isSelected ? COLORS.nodeSelected : isHighlighted ? COLORS.nodeFocal : COLORS.nodeBorder;
      this.ctx.lineWidth = isSelected || isHighlighted ? 2 : 1;
      this.roundRect(nl.x, nl.y, nl.width, nl.height, 6);
      this.ctx.fill();
      this.ctx.stroke();
      this.ctx.globalAlpha = 1;
      this.ctx.font = 'bold 11px Inter, sans-serif';
      this.ctx.fillStyle = isFocal ? COLORS.nodeFocal : COLORS.textPeripheral;
      this.ctx.textAlign = 'center';
      this.ctx.textBaseline = 'top';
      this.ctx.fillText(node.type, nl.x + nl.width / 2, nl.y + 8);
      this.ctx.beginPath();
      this.ctx.strokeStyle = COLORS.nodeBorder;
      this.ctx.lineWidth = 0.5;
      this.ctx.moveTo(nl.x + 8, nl.y + 24);
      this.ctx.lineTo(nl.x + nl.width - 8, nl.y + 24);
      this.ctx.stroke();
      const attrs = this.getDisplayAttributes(node, 3);
      this.ctx.font = '10px Inter, sans-serif';
      this.ctx.textAlign = 'left';
      this.ctx.textBaseline = 'top';
      let yOffset = 30;
      for (const [key, value] of attrs) {
        this.ctx.fillStyle = isFocal ? COLORS.textSecondary : COLORS.textPeripheral;
        const keyText = `${key}: `;
        this.ctx.fillText(keyText, nl.x + 8, nl.y + yOffset);
        const keyWidth = this.ctx.measureText(keyText).width;
        this.ctx.fillStyle = isFocal ? COLORS.textPrimary : COLORS.textPeripheral;
        const valueText = this.truncateText(String(value), nl.width - 16 - keyWidth);
        this.ctx.fillText(valueText, nl.x + 8 + keyWidth, nl.y + yOffset);
        yOffset += 14;
      }
    }
  }

  private getDisplayAttributes(node: NodeData, maxAttrs: number): [string, unknown][] {
    const priorityKeys = ['name', 'title', 'label', 'email', 'status', 'id'];
    const result: [string, unknown][] = [];
    for (const key of priorityKeys) {
      if (key in node.attrs && result.length < maxAttrs) {
        result.push([key, node.attrs[key]]);
      }
    }
    for (const [key, value] of Object.entries(node.attrs)) {
      if (!priorityKeys.includes(key) && result.length < maxAttrs) {
        result.push([key, value]);
      }
    }
    return result;
  }

  private truncateText(text: string, maxWidth: number): string {
    if (this.ctx.measureText(text).width <= maxWidth) return text;
    let truncated = text;
    while (truncated.length > 0 && this.ctx.measureText(truncated + '...').width > maxWidth) {
      truncated = truncated.slice(0, -1);
    }
    return truncated + '...';
  }

  private roundRect(x: number, y: number, w: number, h: number, r: number) {
    this.ctx.beginPath();
    this.ctx.moveTo(x + r, y);
    this.ctx.lineTo(x + w - r, y);
    this.ctx.quadraticCurveTo(x + w, y, x + w, y + r);
    this.ctx.lineTo(x + w, y + h - r);
    this.ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
    this.ctx.lineTo(x + r, y + h);
    this.ctx.quadraticCurveTo(x, y + h, x, y + h - r);
    this.ctx.lineTo(x, y + r);
    this.ctx.quadraticCurveTo(x, y, x + r, y);
    this.ctx.closePath();
  }

}
