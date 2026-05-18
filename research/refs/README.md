# Reference Implementations & Papers

Git submodules for upstream code we study or adapt from. Fetched papers live in `papers/` (gitignored).

## Fetching papers

Run the fetch script to download all reference PDFs and HTML snapshots to `papers/`:

```bash
bash research/refs/fetch-papers.sh
```

Papers fetched:
| Key | Description | Format |
|---|---|---|
| `aloha` | ALOHA helicity amplitude generator | HTML (ar5iv) |
| `ufo` | Universal FeynRules Output format | HTML (ar5iv) |
| `madgraph5` | MadGraph5_aMC@NLO | HTML (ar5iv) |
| `madgraph_orig` | Original MadGraph (Stelzer & Long) | HTML (ar5iv) |
| `helas` | HELAS manual (KEK scanned PDF) | PDF |
| `vegas` | VEGAS+ adaptive importance sampling | HTML (ar5iv) |
| `mcreview` | Monte Carlo methods review | HTML (ar5iv) |

## OCR for scanned PDFs (HELAS)

The HELAS manual (`papers/helas.pdf`) is a scanned document and requires OCR.
We use [Nougat](https://github.com/facebookresearch/nougat) (Meta), which outputs markdown+LaTeX.

The `nougat` pixi environment is configured in `pixi.toml` with the required dependency pins.
Three pins are needed because nougat 0.1.17 predates several breaking upstream changes:

| Package | Pin | Reason |
|---|---|---|
| `albumentations` | `<1.4` | `ImageCompression` API changed (int → string for `compression_type`) |
| `pypdfium2` | `<5` | `PdfDocument.render()` removed in v5 |
| `transformers` | `<4.36` | `cache_position` kwarg added to `generate()`, not handled by nougat |

To run OCR:

```bash
pixi run -e nougat ocr
```

Output is written to `papers/helas.mmd` (markdown+LaTeX). This takes several minutes on CPU;
runs faster with an MPS/CUDA GPU available.

## Adding a submodule

```bash
git submodule add <url> research/refs/<name>
git commit -m "research: add <name> reference"
```

## Suggested References

| Name | URL | Purpose |
|---|---|---|
| `mg5amcnlo` | https://github.com/mg5amcnlo/mg5amcnlo | Diagram generation, ALOHA output |
| `aloha` | (part of mg5amcnlo) | Helicity amplitude generation from UFO |
| `feyngraph` | https://github.com/Jens-Braun/FeynGraph | Rust Feynman diagram crate |
