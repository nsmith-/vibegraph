# Reference Implementations

Git submodules for upstream code we study or adapt from.

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
| `feyngraph` | https://github.com/... | Rust Feynman diagram crate |
