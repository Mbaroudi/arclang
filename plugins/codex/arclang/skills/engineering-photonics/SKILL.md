---
name: engineering-photonics
description: Use when a Fourier transform computed in light is part of an ArcLang model: a radix-2 butterfly network or a reconfigurable Reck mesh of couplers and phase shifters, its precision against phase errors and loss, effective bits, insertion loss.
---

# An optical transform as code

```arc
node "16-point transform" {
  id: "PN-FFT16"
  realized_by: ["photonics:fft16.json"]     // {"network": "fft", "points": 16}
  phase_sigma: "5 mrad"
  loss_per_stage: "2 dB"
  limits: ["relative_error < 2 %", "effective_bits >= 5 bit", "insertion_loss < 30 dB"]
}
```

- `fft`: a radix-2 butterfly, 4 stages of 8 couplers for 16 points. `reck`: a reconfigurable
  mesh of 120 Mach-Zehnder interferometers, longest path 29. Same transform, different
  precision.
- The engine runs 200 trials (seed 1 by default) with Gaussian phase errors and reports the
  95th percentile of the relative error. `effective_bits` in a verdict is `-log2(p95)`.
- Measured with the defaults: the butterfly at 5 mrad gives an error of 0.0087, 6.85 bits,
  8 dB; at 50 mrad 0.0871, 3.52 bits. One bit is lost each time the phase error doubles. The
  mesh at 5 mrad gives 5.02 bits, about 1.8 bits behind the butterfly at equal error.
- A loss per stage is a pure attenuation in the butterfly, but a distortion in the mesh: with
  2 dB per stage the butterfly keeps 6.85 bits and the mesh drops to a relative error of 1.59.
- Credibility checks run before any verdict: ideal equality to 1e-9, coupler and butterfly
  against the reference to 1e-12.

**What it is not.** A circuit-level model of ideal elements with the errors you write: no
dispersion, no crosstalk, no detector noise. All precision figures are simulation; no optical
measurement exists behind them. Limits written in place show as `UNSOURCED` data lines.

Online, the same engine runs on arclang.fr for a signed-in user (40 jobs a day).
