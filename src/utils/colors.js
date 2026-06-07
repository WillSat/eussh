export function darkenHex(hex, amount) {
  let r, g, b
  if (hex.startsWith('#')) {
    const h = hex.slice(1)
    if (h.length === 3) {
      r = parseInt(h[0] + h[0], 16)
      g = parseInt(h[1] + h[1], 16)
      b = parseInt(h[2] + h[2], 16)
    } else {
      r = parseInt(h.slice(0, 2), 16)
      g = parseInt(h.slice(2, 4), 16)
      b = parseInt(h.slice(4, 6), 16)
    }
  } else {
    return hex
  }
  r = Math.max(0, Math.min(255, r + amount))
  g = Math.max(0, Math.min(255, g + amount))
  b = Math.max(0, Math.min(255, b + amount))
  return '#' + [r, g, b].map(c => c.toString(16).padStart(2, '0')).join('')
}
