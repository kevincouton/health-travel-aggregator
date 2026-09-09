// Generates the PWA icons in public/icons/ — a white medical cross on the
// brand blue (Tailwind blue-600). Pure Node (zlib only), supersampled for
// clean anti-aliased edges. Re-run with: node scripts/generate-pwa-icons.mjs
import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { deflateSync } from 'node:zlib'

const BRAND = [0x25, 0x63, 0xeb] // #2563eb
const WHITE = [0xff, 0xff, 0xff]
const SUPERSAMPLE = 4

const crcTable = new Uint32Array(256).map((_, n) => {
  let c = n
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
  return c >>> 0
})

function crc32(buf) {
  let c = 0xffffffff
  for (const byte of buf) c = crcTable[(c ^ byte) & 0xff] ^ (c >>> 8)
  return (c ^ 0xffffffff) >>> 0
}

function chunk(type, data) {
  const out = Buffer.alloc(12 + data.length)
  out.writeUInt32BE(data.length, 0)
  out.write(type, 4, 'ascii')
  data.copy(out, 8)
  out.writeUInt32BE(crc32(out.subarray(4, 8 + data.length)), 8 + data.length)
  return out
}

function encodePng(size, rgba) {
  const ihdr = Buffer.alloc(13)
  ihdr.writeUInt32BE(size, 0)
  ihdr.writeUInt32BE(size, 4)
  ihdr[8] = 8 // bit depth
  ihdr[9] = 6 // color type RGBA
  const raw = Buffer.alloc((size * 4 + 1) * size)
  for (let y = 0; y < size; y++) {
    raw[y * (size * 4 + 1)] = 0 // filter: none
    rgba.copy(raw, y * (size * 4 + 1) + 1, y * size * 4, (y + 1) * size * 4)
  }
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ])
}

// Signed distance to a rounded box centered at the origin.
function sdRoundBox(px, py, halfW, halfH, radius) {
  const qx = Math.abs(px) - halfW + radius
  const qy = Math.abs(py) - halfH + radius
  const ax = Math.max(qx, 0)
  const ay = Math.max(qy, 0)
  return Math.hypot(ax, ay) + Math.min(Math.max(qx, qy), 0) - radius
}

function render(size, { cornerRadius, crossLength, crossWidth }) {
  const hi = size * SUPERSAMPLE
  const hiBuf = Buffer.alloc(hi * hi * 4)
  const r = cornerRadius * size
  const halfLen = (crossLength * size) / 2
  const halfWid = (crossWidth * size) / 2
  for (let y = 0; y < hi; y++) {
    for (let x = 0; x < hi; x++) {
      // Sample at the pixel center, coordinates relative to the icon center.
      const px = ((x + 0.5) / hi - 0.5) * size
      const py = ((y + 0.5) / hi - 0.5) * size
      const offset = (y * hi + x) * 4
      if (sdRoundBox(px, py, size / 2, size / 2, r) > 0) continue // transparent
      const cross = Math.min(
        sdRoundBox(px, py, halfWid, halfLen, halfWid),
        sdRoundBox(px, py, halfLen, halfWid, halfWid)
      )
      const [cr, cg, cb] = cross <= 0 ? WHITE : BRAND
      hiBuf[offset] = cr
      hiBuf[offset + 1] = cg
      hiBuf[offset + 2] = cb
      hiBuf[offset + 3] = 255
    }
  }
  // Box-filter downsample to the target size.
  const out = Buffer.alloc(size * size * 4)
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const acc = [0, 0, 0, 0]
      for (let sy = 0; sy < SUPERSAMPLE; sy++) {
        for (let sx = 0; sx < SUPERSAMPLE; sx++) {
          const src = ((y * SUPERSAMPLE + sy) * hi + x * SUPERSAMPLE + sx) * 4
          for (let c = 0; c < 4; c++) acc[c] += hiBuf[src + c]
        }
      }
      const dst = (y * size + x) * 4
      const n = SUPERSAMPLE * SUPERSAMPLE
      for (let c = 0; c < 4; c++) out[dst + c] = Math.round(acc[c] / n)
    }
  }
  return out
}

const outDir = join(dirname(fileURLToPath(import.meta.url)), '..', 'public', 'icons')
mkdirSync(outDir, { recursive: true })

// Standard icons: rounded-square tile. The cross fills 62% of the tile.
const standard = { cornerRadius: 0.22, crossLength: 0.62, crossWidth: 0.24 }
// Maskable icons get cropped to a circle/squircle by the OS, so they are
// full-bleed with the cross inside the 80% safe zone.
const maskable = { cornerRadius: 0, crossLength: 0.5, crossWidth: 0.2 }

for (const [name, size, opts] of [
  ['icon-192.png', 192, standard],
  ['icon-512.png', 512, standard],
  ['maskable-512.png', 512, maskable],
  ['apple-touch-icon.png', 180, maskable],
]) {
  const png = encodePng(size, render(size, opts))
  writeFileSync(join(outDir, name), png)
  console.log(`${name}: ${size}x${size}, ${png.length} bytes`)
}
