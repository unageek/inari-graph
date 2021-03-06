import { BigNumber } from "bignumber.js";
import * as L from "leaflet";
import { BASE_ZOOM_LEVEL } from "./constants";

BigNumber.config({
  EXPONENTIAL_AT: 5,
  // Division is used for inverting mantissas and transform to pixel coordinates,
  // which do not require much precision.
  DECIMAL_PLACES: 2,
});

function bignum(x: number): BigNumber {
  return new BigNumber(x);
}

declare module "bignumber.js" {
  interface BigNumber {
    ceil(): BigNumber;
    floor(): BigNumber;
  }
}

BigNumber.prototype.ceil = function (): BigNumber {
  return this.integerValue(BigNumber.ROUND_CEIL);
};

BigNumber.prototype.floor = function (): BigNumber {
  return this.integerValue(BigNumber.ROUND_FLOOR);
};

const ZERO: BigNumber = bignum(0);
const ONE: BigNumber = bignum(1);
const TILE_SIZE = 256;
const RETINA_SCALE = window.devicePixelRatio;
const LABEL_FONT = "14px 'Noto Sans'";

interface Transform {
  (x: BigNumber): number;
}

class GridInterval {
  x?: BigNumber;
  xInv?: BigNumber;

  constructor(readonly mant: BigNumber, readonly exp: number) {}

  get(): BigNumber {
    return (this.x ??= this.mant.times(ONE.shiftedBy(this.exp)));
  }

  getInv(): BigNumber {
    return (this.xInv ??= ONE.div(this.mant).times(ONE.shiftedBy(-this.exp)));
  }
}

/**
 * Returns the 1D affine transformation that maps points xs to ys.
 * @param srcPoints A pair of source points.
 * @param dstPoints A pair of destination points.
 */
function getTransform(
  srcPoints: [BigNumber, BigNumber],
  dstPoints: [BigNumber, BigNumber]
): Transform {
  const [x0, x1] = srcPoints;
  const [y0, y1] = dstPoints;
  const d = x1.minus(x0);
  const a = y1.minus(y0);
  const b = x1.times(y0).minus(x0.times(y1));
  return (x) => {
    return +a.times(x).plus(b).div(d);
  };
}

const mantissas = [1, 2, 5].map(bignum);
/**
 * Returns grid interval.
 * @param widthPerPixel The width of a pixel in the real coordinates.
 * @returns The pair of the major and minor grid intervals.
 */
function gridIntervals(widthPerPixel: number): [GridInterval, GridInterval] {
  function interval(level: number): GridInterval {
    const e = Math.floor(level / 3);
    const m = mantissas[level - 3 * e];
    return new GridInterval(m, e);
  }

  const maxDensity = 20; // One minor line per 20px at most.
  const e = Math.floor(Math.log10(widthPerPixel * maxDensity)) - 1;
  let level = 3 * e;
  for (;;) {
    const minInterval = interval(level);
    if (+minInterval.get() / widthPerPixel >= maxDensity) {
      return [interval(level + 2), minInterval];
    }
    level++;
  }
}

const dstX: [BigNumber, BigNumber] = [bignum(0.5), bignum(TILE_SIZE + 0.5)];
const dstY: [BigNumber, BigNumber] = [bignum(TILE_SIZE + 0.5), bignum(0.5)];
class StaticGridLayer extends L.GridLayer {
  protected createTile(coords: L.Coords, done: L.DoneCallback): HTMLElement {
    const tile = L.DomUtil.create(
      "canvas",
      "leaflet-tile"
    ) as HTMLCanvasElement;
    tile.width = RETINA_SCALE * TILE_SIZE;
    tile.height = RETINA_SCALE * TILE_SIZE;
    tile.style.width = TILE_SIZE + "px";
    tile.style.height = TILE_SIZE + "px";

    setTimeout(() => {
      const widthPerTilef = 2 ** (BASE_ZOOM_LEVEL - coords.z);
      const widthPerTile = bignum(widthPerTilef);
      const x0 = widthPerTile.times(bignum(coords.x));
      const x1 = widthPerTile.times(bignum(coords.x + 1));
      const y0 = widthPerTile.times(bignum(-coords.y - 1));
      const y1 = widthPerTile.times(bignum(-coords.y));
      const tx = getTransform([x0, x1], dstX);
      const ty = getTransform([y0, y1], dstY);

      const widthPerPixel = widthPerTilef / TILE_SIZE;
      const [majInterval, minInterval] = gridIntervals(widthPerPixel);

      const ctx = tile.getContext("2d")!;
      ctx.setTransform(RETINA_SCALE, 0, 0, RETINA_SCALE, 0, 0);

      ctx.fillStyle = "white";
      ctx.fillRect(0, 0, TILE_SIZE, TILE_SIZE);

      ctx.strokeStyle = "#e0e0e0";
      this.drawGrid(ctx, x0, y0, x1, y1, minInterval, tx, ty);

      ctx.strokeStyle = "#c0c0c0";
      this.drawGrid(ctx, x0, y0, x1, y1, majInterval, tx, ty);

      ctx.strokeStyle = "black";
      this.drawAxes(ctx, tx, ty);

      done(undefined, tile);
    }, 0);

    return tile;
  }

  private drawAxes(
    ctx: CanvasRenderingContext2D,
    tx: Transform,
    ty: Transform
  ) {
    const cx = tx(ZERO);
    const cy = ty(ZERO);
    ctx.beginPath();
    ctx.moveTo(cx, 0);
    ctx.lineTo(cx, TILE_SIZE);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(0, cy);
    ctx.lineTo(TILE_SIZE, cy);
    ctx.stroke();
  }

  private drawGrid(
    ctx: CanvasRenderingContext2D,
    x0: BigNumber,
    y0: BigNumber,
    x1: BigNumber,
    y1: BigNumber,
    interval: GridInterval,
    tx: Transform,
    ty: Transform
  ) {
    ctx.beginPath();
    {
      const min = x0.times(interval.getInv()).ceil().minus(ONE);
      const max = x1.times(interval.getInv()).floor().plus(ONE);
      for (let i = min; i.lte(max); i = i.plus(ONE)) {
        const x = i.times(interval.get());
        const cx = tx(x);
        ctx.moveTo(cx, 0);
        ctx.lineTo(cx, TILE_SIZE);
      }
    }
    {
      const min = y0.times(interval.getInv()).ceil().minus(ONE);
      const max = y1.times(interval.getInv()).floor().plus(ONE);
      for (let i = min; i.lte(max); i = i.plus(ONE)) {
        const y = i.times(interval.get());
        const cy = ty(y);
        ctx.moveTo(0, cy);
        ctx.lineTo(TILE_SIZE, cy);
      }
    }
    ctx.stroke();
  }
}

class GridLabelsLayer extends L.GridLayer {
  /// Distance between the axes and tick labels.
  readonly labelOffset = 4;
  /// Distance between the map boundary and tick labels.
  readonly padding = 4;

  onAdd(map: L.Map): this {
    super.onAdd(map);
    map.on("move", this.redrawCurrentTiles, this);
    return this;
  }

  onRemove(map: L.Map): this {
    super.onRemove(map);
    map.off("move", this.redrawCurrentTiles, this);
    return this;
  }

  protected createTile(coords: L.Coords, done: L.DoneCallback): HTMLElement {
    const tile = L.DomUtil.create(
      "canvas",
      "leaflet-tile"
    ) as HTMLCanvasElement;
    tile.width = RETINA_SCALE * TILE_SIZE;
    tile.height = RETINA_SCALE * TILE_SIZE;
    tile.style.width = TILE_SIZE + "px";
    tile.style.height = TILE_SIZE + "px";

    document.fonts.load(LABEL_FONT).then(() => {
      const tileRange = this.getVisibleTileRange();
      this.drawTile(tile, coords, tileRange);
      done(undefined, tile);
    });

    return tile;
  }

  private drawOriginLabel(
    ctx: CanvasRenderingContext2D,
    tx: Transform,
    ty: Transform
  ) {
    const cx = tx(ZERO);
    const cy = ty(ZERO);
    const text = "0";
    const m = ctx.measureText(text);
    const args: [string, number, number] = [
      text,
      cx - m.actualBoundingBoxRight - this.labelOffset,
      cy + m.actualBoundingBoxAscent + this.labelOffset,
    ];
    ctx.fillStyle = "black";
    ctx.strokeText(...args);
    ctx.fillText(...args);
  }

  private drawXTickLabels(
    ctx: CanvasRenderingContext2D,
    x0: BigNumber,
    x1: BigNumber,
    interval: GridInterval,
    tx: Transform,
    ty: Transform,
    mapViewport: DOMRect,
    tileViewport: DOMRect
  ) {
    {
      const cy = ty(ZERO);
      const min = x0.times(interval.getInv()).ceil().minus(ONE);
      const max = x1.times(interval.getInv()).floor().plus(ONE);
      for (let i = min; i.lte(max); i = i.plus(ONE)) {
        if (i.isZero()) continue;
        const x = i.times(interval.get());
        const cx = tx(x);
        const wx = tileViewport.x + cx;
        if (wx < mapViewport.left || wx > mapViewport.right) {
          continue;
        }
        const text = this.format(x);
        const m = ctx.measureText(text);
        const args: [string, number, number] = [
          text,
          cx - (m.actualBoundingBoxLeft + m.actualBoundingBoxRight) / 2,
          cy + m.actualBoundingBoxAscent + this.labelOffset,
        ];
        const textBounds = this.getBoundingRect(ctx, ...args, tileViewport);
        const args2: [string, number, number] = [
          text,
          args[1] +
            Math.max(0, mapViewport.left + this.padding - textBounds.left) +
            Math.min(0, mapViewport.right - this.padding - textBounds.right),
          args[2] +
            Math.max(0, mapViewport.top + this.padding - textBounds.top) +
            Math.min(0, mapViewport.bottom - this.padding - textBounds.bottom),
        ];
        ctx.fillStyle = args2[2] !== args[2] ? "gray" : "black";
        ctx.strokeText(...args2);
        ctx.fillText(...args2);
      }
    }
  }

  private drawYTickLabels(
    ctx: CanvasRenderingContext2D,
    y0: BigNumber,
    y1: BigNumber,
    interval: GridInterval,
    tx: Transform,
    ty: Transform,
    mapViewport: DOMRect,
    tileViewport: DOMRect
  ) {
    const cx = tx(ZERO);
    const min = y0.times(interval.getInv()).ceil().minus(ONE);
    const max = y1.times(interval.getInv()).floor().plus(ONE);
    for (let i = min; i.lte(max); i = i.plus(ONE)) {
      if (i.isZero()) continue;
      const y = i.times(interval.get());
      const cy = ty(y);
      const wy = tileViewport.y + cy;
      if (wy < mapViewport.top || wy > mapViewport.bottom) {
        continue;
      }
      const text = this.format(y);
      const m = ctx.measureText(text);
      const args: [string, number, number] = [
        text,
        cx - m.actualBoundingBoxRight - this.labelOffset,
        cy + (m.actualBoundingBoxAscent - m.actualBoundingBoxDescent) / 2,
      ];
      const textBounds = this.getBoundingRect(ctx, ...args, tileViewport);
      const args2: [string, number, number] = [
        text,
        args[1] +
          Math.max(0, mapViewport.left + this.padding - textBounds.left) +
          Math.min(0, mapViewport.right - this.padding - textBounds.right),
        args[2] +
          Math.max(0, mapViewport.top + this.padding - textBounds.top) +
          Math.min(0, mapViewport.bottom - this.padding - textBounds.bottom),
      ];
      ctx.fillStyle = args2[1] !== args[1] ? "gray" : "black";
      ctx.strokeText(...args2);
      ctx.fillText(...args2);
    }
  }

  private drawTile(
    tile: HTMLCanvasElement,
    coords: L.Coords,
    tileRange: L.Bounds
  ) {
    const widthPerTilef = 2 ** (BASE_ZOOM_LEVEL - coords.z);
    const widthPerTile = bignum(widthPerTilef);
    const x0 = widthPerTile.times(bignum(coords.x));
    const x1 = widthPerTile.times(bignum(coords.x + 1));
    const y0 = widthPerTile.times(bignum(-coords.y - 1));
    const y1 = widthPerTile.times(bignum(-coords.y));
    const tx = getTransform([x0, x1], dstX);
    const ty = getTransform([y0, y1], dstY);

    const widthPerPixel = widthPerTilef / TILE_SIZE;
    const [interval] = gridIntervals(widthPerPixel);

    const ctx = tile.getContext("2d")!;
    ctx.setTransform(RETINA_SCALE, 0, 0, RETINA_SCALE, 0, 0);
    const mapViewport = this._map.getContainer().getBoundingClientRect();
    const tileViewport = ctx.canvas.getBoundingClientRect();
    ctx.clearRect(0, 0, TILE_SIZE, TILE_SIZE);
    ctx.font = LABEL_FONT;
    ctx.lineJoin = "round";
    ctx.lineWidth = 3;
    ctx.strokeStyle = "white";
    if (coords.x === -1 && coords.y === 0) {
      this.drawOriginLabel(ctx, tx, ty);
    }
    if (
      coords.y === 0 ||
      (tileRange.max!.y <= 0 &&
        (coords.y === tileRange.max!.y || coords.y === tileRange.max!.y - 1)) ||
      (tileRange.min!.y >= 0 &&
        (coords.y === tileRange.min!.y || coords.y === tileRange.min!.y + 1))
    ) {
      this.drawXTickLabels(
        ctx,
        x0,
        x1,
        interval,
        tx,
        ty,
        mapViewport,
        tileViewport
      );
    }
    if (
      coords.x === -1 ||
      (tileRange.max!.x <= -1 &&
        (coords.x === tileRange.max!.x || coords.x === tileRange.max!.x - 1)) ||
      (tileRange.min!.x >= -1 &&
        (coords.x === tileRange.min!.x || coords.x === tileRange.min!.x + 1))
    ) {
      this.drawYTickLabels(
        ctx,
        y0,
        y1,
        interval,
        tx,
        ty,
        mapViewport,
        tileViewport
      );
    }
  }

  private format(x: BigNumber) {
    return x.toString().replaceAll("-", "−");
  }

  private getBoundingRect(
    ctx: CanvasRenderingContext2D,
    text: string,
    cx: number,
    cy: number,
    tileViewport: DOMRect
  ) {
    const m = ctx.measureText(text);
    return new DOMRect(
      tileViewport.x + cx - m.actualBoundingBoxLeft,
      tileViewport.y + cy - m.actualBoundingBoxAscent,
      m.actualBoundingBoxLeft + m.actualBoundingBoxRight,
      m.actualBoundingBoxAscent + m.actualBoundingBoxDescent
    );
  }

  private getVisibleTileRange(): L.Bounds {
    const bounds = this._map.getPixelBounds();
    return new L.Bounds(
      new L.Point(
        Math.floor(bounds.min!.x / TILE_SIZE),
        Math.floor(bounds.min!.y / TILE_SIZE)
      ),
      new L.Point(
        Math.ceil((bounds.max!.x - (TILE_SIZE - 1)) / TILE_SIZE),
        Math.ceil((bounds.max!.y - (TILE_SIZE - 1)) / TILE_SIZE)
      )
    );
  }

  private redrawCurrentTiles() {
    const tileRange = this.getVisibleTileRange();
    // https://github.com/Leaflet/Leaflet/blob/436430db4203a350601e002c8de6a41fae15a4bf/src/layer/tile/GridLayer.js#L318
    for (const key in this._tiles) {
      const tile = this._tiles[key];
      if (!tile.current || !tile.loaded) {
        continue;
      }
      this.drawTile(tile.el as HTMLCanvasElement, tile.coords, tileRange);
    }
  }
}

export class GridLayer extends L.LayerGroup {
  constructor(options?: L.LayerOptions) {
    super([new StaticGridLayer(), new GridLabelsLayer()], options);
  }
}
