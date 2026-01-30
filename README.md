# engulf

`engulf` is a small command-line utility that analyses a JSON document and
converts it into a set of *folded stack* lines that can be fed to any standard
flame graph visualiser (e.g. Brendan Gregg’s `flamegraph.pl` or
[speedscope.app](https://www.speedscope.app/)).  This lets you see the shape
and size of your JSON at a glance – perfect for spotting the largest objects
or simply getting an overview of deeply nested data.

## CLI usage

```text
Usage: engulf [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input JSON file

Options:
  -o, --output <OUTPUT>    Output file (stdout if omitted)
      --group-by <KEY>...  Group array elements (objects) by one or more keys
  -h, --help               Print help
  -V, --version            Print version
```


### Quick example

```bash
# Analyse `data.json`, grouping array elements by their `type` field, and
# write the folded-stack output to `data.folded`.

engulf data.json --group-by type -o data.folded

# Now turn the folded stacks into an interactive flame graph (SVG) with
# Brendan Gregg’s FlameGraph tools (https://github.com/brendangregg/FlameGraph):
flamegraph.pl data.folded > data.svg

# …or drag & drop `data.folded` into https://www.speedscope.app/
```

This is the svg produced from a temporal agent run. Open the svg in your browser to interact.
![Agent run workflow byte flamegraph](images/out.svg)


## Viewing the results

- **Brendan Gregg’s FlameGraph** (Perl scripts): converts folded stacks into
  static SVG flame graphs that you can open in any browser.
- **speedscope.app**: a web-based viewer that lets you zoom, filter and search
  through large profiles interactively. Open the site and drop the
  `*.folded` file onto the page.

