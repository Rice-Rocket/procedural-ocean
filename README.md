# Ocean Simulation

## Description

An ocean simulation written in [Rust](https://www.rust-lang.org/) with [Bevy](https://bevyengine.org/) and [Wgpu](https://wgpu.rs/). 

### Features

- Ocean height displacement calculated using an inverse FFT on the JONSWAP spectrum
- All computation done in parallel on the gpu
- Realistic lighting model with subsurface scattering
- Foam accumulated based on water turbulence

## References

- ["Simulation Ocean Water" by Jerry Tessendorf](https://people.computing.clemson.edu/~jtessen/reports/papers_files/coursenotes2004.pdf)
- ["Technical Art of Sea of Thieves" Siggraph talk](https://youtu.be/y9BOz2dFZzs?si=mEPPMDdY4_9WJu9R)