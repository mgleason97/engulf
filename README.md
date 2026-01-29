# Engulf

Engulf is a small command-line utility that analyses a JSON document and
converts it into a set of *folded stack* lines that can be fed to any standard
flame graph visualiser (e.g. Brendan Gregg’s `flamegraph.pl` or
[speedscope.app](https://www.speedscope.app/)).  This lets you **see the shape
and size of your JSON** at a glance – perfect for spotting the largest objects
or simply getting an overview of deeply nested data.

## CLI usage

```text
engulf <INPUT_JSON> [OPTIONS]

Arguments:
  <INPUT_JSON>            Path to the JSON file to analyse

Options:
  -o, --output <FILE>     Write folded stacks to <FILE> (stdout if omitted)
      --group-by <KEY>    For arrays of JSON objects, group elements that
                          share the same value for <KEY>
  -h, --help              Show help
  -V, --version           Show version information
```


### Quick example

```bash
# Analyse `profile.json`, grouping array elements by their `type` field, and
# write the folded-stack output to `profile.folded`.

engulf profile.json --group-by type -o profile.folded

# Now turn the folded stacks into an interactive flame graph (SVG) with
# Brendan Gregg’s FlameGraph tools (https://github.com/brendangregg/FlameGraph):
flamegraph.pl profile.folded > profile.svg

# …or drag & drop `profile.folded` into https://www.speedscope.app/
```

This is the svg produced from a temporal agent run. Open the svg in your browser to interact.
![Agent run workflow byte flamegraph](images/out.svg)


## Viewing the results

- **Brendan Gregg’s FlameGraph** (Perl scripts): converts folded stacks into
  static SVG flame graphs that you can open in any browser.
- **speedscope.app**: a web-based viewer that lets you zoom, filter and search
  through large profiles interactively. Open the site and drop the
  `*.folded` file onto the page.

