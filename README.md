# <img src="https://www.rust-lang.org/logos/rust-logo-blk.svg" width="100"> Rust template ![GitHub License](https://img.shields.io/github/license/MathieuSoysal/Exercism-Rust-Template)
[![Static Badge](https://img.shields.io/badge/online-green?logo=gamejolt&logoColor=white&label=Benchmark%20tracks&labelColor=black&link=https%3A%2F%2Fbencher.dev%2Fconsole%2Fprojects%2Frust-template)](https://bencher.dev/console/projects/rust-template)
[![codecov](https://codecov.io/gh/MathieuSoysal/Exercism-Rust-Template/graph/badge.svg?token=MrM1EEfgvD)](https://codecov.io/gh/MathieuSoysal/Exercism-Rust-Template)
[![Integration test for Rust](https://github.com/MathieuSoysal/Exercism-Rust-Template/actions/workflows/integration-test.yml/badge.svg)](https://github.com/MathieuSoysal/Exercism-Rust-Template/actions/workflows/integration-test.yml)


Basic Rust template with rust devcontainer and some CI to test and track benchmarcks automatically.

## Resources

- Benchmark tool : https://bencher.dev/learn/benchmarking/rust/libtest-bench/
- Benchmark CI : https://bencher.dev/docs/how-to/github-actions/
- mutation testing : https://mutants.rs

## Customizing template

### Ctrl + Shift + F for replace:
- [ ] `template-exercisme` -> with your rust project name
- [ ] `Exercism-Rust-Template` -> with your GitHub repository name
- [ ] `rust-template` -> with your Bencher project name
- [ ] `MathieuSoysal` -> with your GitHub username
- [ ] `2024` -> with the current year

### Secrets
- [ ] Add `BENCHER_API_KEY` in your repository secrets
- [ ] Add `CODECOV_TOKEN` in your repository secrets