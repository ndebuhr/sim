<div align="center">
	<img src="https://simrs.com/images/logo.png" width="200" height="200">
	<h1>Sim</h1>
  <p>"Sim" or "SimRS" is a discrete event simulation package that facilitates<br>Rust- and npm-based simulation products and projects</p>
  <p><a href="https://simrs.com">Sim Website</a> | <a href="https://simrs.com/demo/">Sim Demo</a> | <a href="https://docs.rs/sim/">Sim Docs</p>
  <br>
</div>

![stability-experimental](https://img.shields.io/badge/stability-experimental-bd0058.svg?style=flat-square)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ndebuhr/sim/CI?style=flat-square)](https://github.com/ndebuhr/sim/actions)
[![Crates.io](https://img.shields.io/crates/v/sim?style=flat-square)](https://crates.io/crates/sim)
![Crates.io](https://img.shields.io/crates/d/sim?label=crate%20downloads&style=flat-square)
[![npm](https://img.shields.io/npm/v/sim-rs?style=flat-square)](https://www.npmjs.com/package/sim-rs)
![npm](https://img.shields.io/npm/dt/sim-rs?label=npm%20downloads&style=flat-square)
[![docs.rs](https://img.shields.io/badge/docs.rs-sim-purple?style=flat-square)](https://docs.rs/sim/)
[![Codecov](https://img.shields.io/codecov/c/github/ndebuhr/sim?style=flat-square)](https://codecov.io/gh/ndebuhr/sim)
[![Crates.io](https://img.shields.io/crates/l/sim?style=flat-square)](#license)

"Sim" or "SimRS" is a discrete event simulation package that facilitates Rust- and npm-based simulation products and projects.

This repository contains:

1. [Random variable framework](/sim/src/input_modeling), for easy specification of stochastic model behaviors.
2. [Out-of-the-box models](/sim/src/models), for quickly building out simulations of dynamic systems with common modular components.
3. [Output analysis framework](/sim/src/output_analysis), for analyzing simulation outputs statistically.
4. [Simulator engine](/sim/src/simulator), for managing and executing discrete event simulations.
5. [Custom model macros](/sim_derive/src), for seamlessly integrating custom models into simulations.

Sim is compatible with a wide variety of compilation targets, including WebAssembly.  Sim does not require nightly Rust.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Background

Simulation is a powerful tool for analyzing and designing complex systems.  However, most simulators have steep learning curves, are proprietary, and suffer from limited portability.  Sim aspires to reduce the time required to build new simulation products, complete simulation projects, and learn simulation fundamentals.  Sim is open source and, by virtue of compilation target flexibility, relatively portable.

## Install

For use in Rust code bases, leverage the package as a `cargo` dependency

```toml
[dependencies]
sim = "0.11"
```

For use as a WebAssembly module in a JavaScript/TypeScript code base, leverage the package as a `npm` dependency

```bash
npm i sim-rs
```

## Usage

Rust simulations are created by passing `Model`s and `Connector`s to `Simulation`'s `post` constructor.  WebAssembly simulations are defined in a declarative YAML or JSON format, and then ingested through `WebSimulation`'s `post_yaml` or `post_json` constructors.  Both models and connectors are required to define the simulation.  For descriptions of the out-of-the-box models, see [MODELS.md](/MODELS.md).

Simulations may be stepped with the `step`, `step_n`, and `step_until` methods.  Input injection is possible with the `inject_input` method.

Analyzing simulations will typically involve some combination of processing model records, collecting message transfers, and using output analysis tools.  Analysis of IID samples and time series data are possible.

Please refer to the documentation at [https://docs.rs/sim](https://docs.rs/sim).  Also, the [test simulations](/sim/tests) are a good reference for creating, running, and analyzing simulations with Sim.

## Contributing

Issues, feature requests, and pull requests are always welcome!

## License

This project is licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT License](https://opensource.org/licenses/MIT) at your option.

[Apache License, Version 2.0](LICENSE-APACHE)

[MIT License](LICENSE-MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in sim by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.