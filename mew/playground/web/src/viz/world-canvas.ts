/**
 * WorldCanvas - Infinite navigable canvas space
 *
 * Provides a 2D world coordinate system with pan/zoom navigation.
 * Content is rendered within this world space at specified positions.
 * Navigation is always available regardless of content.
 */

export interface WorldTransform {
  /** Viewport offset in world coordinates */
  panX: number;
  panY: number;
  /** Zoom level (1 = 100%) */
  zoom: number;
}

export interface WorldCanvasOptions {
  canvas: HTMLCanvasElement;
  /** Background color of the world */
  backgroundColor?: string;
  /** Grid settings (null to disable) */
  grid?: {
    size: number;
    color: string;
    opacity: number;
  } | null;
  /** Zoom constraints */
  minZoom?: number;
  maxZoom?: number;
  /** Callbacks */
  onTransformChange?: (transform: WorldTransform) => void;
  onWorldClick?: (worldX: number, worldY: number, screenX: number, screenY: number) => void;
}

const DEFAULT_BG = '#0a0a0a';
const DEFAULT_GRID = {
  size: 50,
  color: '#1a1a1a',
  opacity: 1,
};

/**
 * WorldCanvas manages an infinite 2D canvas space with pan/zoom navigation.
 */
export class WorldCanvas {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private options: Required<Omit<WorldCanvasOptions, 'canvas'>>;

  // Transform state
  private panX = 0;
  private panY = 0;
  private zoom = 1;

  // Interaction state
  private isDragging = false;
  private lastMouseX = 0;
  private lastMouseY = 0;

  // Content render callback
  private contentRenderer: ((ctx: CanvasRenderingContext2D, transform: WorldTransform) => void) | null = null;

  constructor(options: WorldCanvasOptions) {
    this.canvas = options.canvas;
    this.ctx = this.canvas.getContext('2d')!;

    this.options = {
      backgroundColor: options.backgroundColor ?? DEFAULT_BG,
      grid: options.grid === undefined ? DEFAULT_GRID : options.grid,
      minZoom: options.minZoom ?? 0.1,
      maxZoom: options.maxZoom ?? 5,
      onTransformChange: options.onTransformChange ?? (() => {}),
      onWorldClick: options.onWorldClick ?? (() => {}),
    };

    this.setupEventListeners();
  }

  private setupEventListeners() {
    this.canvas.addEventListener('mousedown', this.handleMouseDown);
    this.canvas.addEventListener('mousemove', this.handleMouseMove);
    this.canvas.addEventListener('mouseup', this.handleMouseUp);
    this.canvas.addEventListener('mouseleave', this.handleMouseUp);
    this.canvas.addEventListener('wheel', this.handleWheel, { passive: false });
    this.canvas.addEventListener('click', this.handleClick);
  }

  destroy() {
    this.canvas.removeEventListener('mousedown', this.handleMouseDown);
    this.canvas.removeEventListener('mousemove', this.handleMouseMove);
    this.canvas.removeEventListener('mouseup', this.handleMouseUp);
    this.canvas.removeEventListener('mouseleave', this.handleMouseUp);
    this.canvas.removeEventListener('wheel', this.handleWheel);
    this.canvas.removeEventListener('click', this.handleClick);
  }

  private handleMouseDown = (e: MouseEvent) => {
    // Middle mouse or left mouse for pan
    if (e.button === 0 || e.button === 1) {
      this.isDragging = true;
      this.lastMouseX = e.clientX;
      this.lastMouseY = e.clientY;
      this.canvas.style.cursor = 'grabbing';
    }
  };

  private handleMouseMove = (e: MouseEvent) => {
    if (this.isDragging) {
      const dx = e.clientX - this.lastMouseX;
      const dy = e.clientY - this.lastMouseY;

      // Pan in screen space, convert to world space
      this.panX += dx / this.zoom;
      this.panY += dy / this.zoom;

      this.lastMouseX = e.clientX;
      this.lastMouseY = e.clientY;

      this.notifyTransformChange();
      this.render();
    } else {
      this.canvas.style.cursor = 'grab';
    }
  };

  private handleMouseUp = () => {
    if (this.isDragging) {
      this.isDragging = false;
      this.canvas.style.cursor = 'grab';
    }
  };

  private handleWheel = (e: WheelEvent) => {
    e.preventDefault();

    const rect = this.canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // Get world position under mouse before zoom
    const worldBeforeX = (mouseX / this.zoom) - this.panX;
    const worldBeforeY = (mouseY / this.zoom) - this.panY;

    // Apply zoom
    const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(
      this.options.minZoom,
      Math.min(this.options.maxZoom, this.zoom * zoomFactor)
    );

    if (newZoom !== this.zoom) {
      this.zoom = newZoom;

      // Adjust pan so world position under mouse stays the same
      this.panX = (mouseX / this.zoom) - worldBeforeX;
      this.panY = (mouseY / this.zoom) - worldBeforeY;

      this.notifyTransformChange();
      this.render();
    }
  };

  private handleClick = (e: MouseEvent) => {
    if (this.isDragging) return;

    const rect = this.canvas.getBoundingClientRect();
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;
    const [worldX, worldY] = this.screenToWorld(screenX, screenY);

    this.options.onWorldClick(worldX, worldY, screenX, screenY);
  };

  private notifyTransformChange() {
    this.options.onTransformChange({
      panX: this.panX,
      panY: this.panY,
      zoom: this.zoom,
    });
  }

  /**
   * Convert screen coordinates to world coordinates
   */
  screenToWorld(screenX: number, screenY: number): [number, number] {
    const worldX = (screenX / this.zoom) - this.panX;
    const worldY = (screenY / this.zoom) - this.panY;
    return [worldX, worldY];
  }

  /**
   * Convert world coordinates to screen coordinates
   */
  worldToScreen(worldX: number, worldY: number): [number, number] {
    const screenX = (worldX + this.panX) * this.zoom;
    const screenY = (worldY + this.panY) * this.zoom;
    return [screenX, screenY];
  }

  /**
   * Set the content renderer callback
   */
  setContentRenderer(renderer: (ctx: CanvasRenderingContext2D, transform: WorldTransform) => void) {
    this.contentRenderer = renderer;
  }

  /**
   * Get current transform
   */
  getTransform(): WorldTransform {
    return { panX: this.panX, panY: this.panY, zoom: this.zoom };
  }

  /**
   * Set transform programmatically
   */
  setTransform(transform: Partial<WorldTransform>) {
    if (transform.panX !== undefined) this.panX = transform.panX;
    if (transform.panY !== undefined) this.panY = transform.panY;
    if (transform.zoom !== undefined) {
      this.zoom = Math.max(this.options.minZoom, Math.min(this.options.maxZoom, transform.zoom));
    }
    this.notifyTransformChange();
    this.render();
  }

  /**
   * Center the view on a world position
   */
  centerOn(worldX: number, worldY: number) {
    this.panX = (this.canvas.width / 2 / this.zoom) - worldX;
    this.panY = (this.canvas.height / 2 / this.zoom) - worldY;
    this.notifyTransformChange();
    this.render();
  }

  /**
   * Fit a bounding box in view
   */
  fitBounds(x: number, y: number, width: number, height: number, padding = 50) {
    const scaleX = (this.canvas.width - padding * 2) / width;
    const scaleY = (this.canvas.height - padding * 2) / height;
    this.zoom = Math.min(scaleX, scaleY, this.options.maxZoom);

    const centerX = x + width / 2;
    const centerY = y + height / 2;
    this.panX = (this.canvas.width / 2 / this.zoom) - centerX;
    this.panY = (this.canvas.height / 2 / this.zoom) - centerY;

    this.notifyTransformChange();
    this.render();
  }

  /**
   * Resize the canvas to fit its container
   */
  resize() {
    const rect = this.canvas.parentElement?.getBoundingClientRect();
    if (rect) {
      this.canvas.width = rect.width;
      this.canvas.height = rect.height;
      this.render();
    }
  }

  /**
   * Main render method
   */
  render() {
    const ctx = this.ctx;
    const { width, height } = this.canvas;

    // Clear with background
    ctx.fillStyle = this.options.backgroundColor;
    ctx.fillRect(0, 0, width, height);

    // Draw grid if enabled
    if (this.options.grid) {
      this.drawGrid();
    }

    // Apply world transform
    ctx.save();
    ctx.scale(this.zoom, this.zoom);
    ctx.translate(this.panX, this.panY);

    // Render content in world space
    if (this.contentRenderer) {
      this.contentRenderer(ctx, this.getTransform());
    }

    ctx.restore();

    // Draw origin marker for reference
    this.drawOriginMarker();
  }

  private drawGrid() {
    const ctx = this.ctx;
    const grid = this.options.grid!;
    const { width, height } = this.canvas;

    // Calculate grid bounds in world space
    const [worldLeft, worldTop] = this.screenToWorld(0, 0);
    const [worldRight, worldBottom] = this.screenToWorld(width, height);

    const startX = Math.floor(worldLeft / grid.size) * grid.size;
    const startY = Math.floor(worldTop / grid.size) * grid.size;

    ctx.save();
    ctx.strokeStyle = grid.color;
    ctx.globalAlpha = grid.opacity;
    ctx.lineWidth = 1 / this.zoom; // Keep line width constant on screen

    ctx.scale(this.zoom, this.zoom);
    ctx.translate(this.panX, this.panY);

    ctx.beginPath();

    // Vertical lines
    for (let x = startX; x <= worldRight; x += grid.size) {
      ctx.moveTo(x, worldTop);
      ctx.lineTo(x, worldBottom);
    }

    // Horizontal lines
    for (let y = startY; y <= worldBottom; y += grid.size) {
      ctx.moveTo(worldLeft, y);
      ctx.lineTo(worldRight, y);
    }

    ctx.stroke();
    ctx.restore();
  }

  private drawOriginMarker() {
    const [screenX, screenY] = this.worldToScreen(0, 0);

    // Only draw if origin is visible
    if (screenX < -20 || screenX > this.canvas.width + 20 ||
        screenY < -20 || screenY > this.canvas.height + 20) {
      return;
    }

    const ctx = this.ctx;
    const size = 8;

    ctx.save();
    ctx.strokeStyle = '#444';
    ctx.lineWidth = 1;

    // Cross at origin
    ctx.beginPath();
    ctx.moveTo(screenX - size, screenY);
    ctx.lineTo(screenX + size, screenY);
    ctx.moveTo(screenX, screenY - size);
    ctx.lineTo(screenX, screenY + size);
    ctx.stroke();

    ctx.restore();
  }
}
