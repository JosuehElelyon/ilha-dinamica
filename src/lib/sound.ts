// Sintetizador leve de alerta sonoro via Web Audio API (sem arquivos externos)
let ctx: AudioContext | null = null

export function playAlertSound() {
  try {
    const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext
    if (!AudioCtx) return
    if (!ctx) ctx = new AudioCtx()
    if (ctx.state === 'suspended') void ctx.resume()

    const now = ctx.currentTime
    // Dois grupos de 3 notas suaves (C5 -> E5 -> G5) para ficar bem audível
    const notes = [523.25, 659.25, 783.99, 523.25, 659.25, 783.99]
    const offsets = [0, 0.14, 0.28, 0.55, 0.69, 0.83]

    notes.forEach((freq, i) => {
      if (!ctx) return
      const osc = ctx.createOscillator()
      const gain = ctx.createGain()
      osc.type = 'sine'
      osc.frequency.setValueAtTime(freq, now + offsets[i])

      gain.gain.setValueAtTime(0.0001, now + offsets[i])
      gain.gain.exponentialRampToValueAtTime(0.22, now + offsets[i] + 0.02)
      gain.gain.exponentialRampToValueAtTime(0.0001, now + offsets[i] + 0.22)

      osc.connect(gain)
      gain.connect(ctx.destination)
      osc.start(now + offsets[i])
      osc.stop(now + offsets[i] + 0.24)
    })
  } catch {
    // ignora se o navegador/sistema bloquear áudio
  }
}
