use std::{
    collections::BTreeMap,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::exit,
};

use anyhow::{Context as _, Result};
use clap::{self, Parser};
use graph::{Edge, Graph};
use itertools::Itertools;
use log::{debug, warn};
use tempfile::NamedTempFile;

use crate::consts::DEFAULT_MAX_DEPTH;

mod bfs;
mod consts;
mod dfs;
mod graph;
#[macro_use]
mod macros;
mod tsort;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Subcommand {
    Show(ShowArgs),
    Dfs(DfsArgs),
    Bfs(BfsArgs),
    Tsort(TsortArgs),
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum InputFormat {
    Text,
    Json,
}

impl InputFormat {
    fn assume_from_path(p: &Path) -> InputFormat {
        let Some(ext) = p.extension().map(|v| v.to_ascii_lowercase().to_string_lossy().to_string()) else { return InputFormat::Text };
        match ext.as_str() {
            "json" => InputFormat::Json,
            _ => InputFormat::Text,
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Dot,
}

impl OutputFormat {
    fn assume_from_path(p: &Path) -> OutputFormat {
        let Some(ext) = p.extension().map(|v| v.to_ascii_lowercase().to_string_lossy().to_string()) else { return OutputFormat::Text };
        match ext.as_str() {
            "json" => OutputFormat::Json,
            "dot" => OutputFormat::Dot,
            _ => OutputFormat::Text,
        }
    }
}

#[derive(Debug, Clone, clap::Args)]
struct ShowArgs {
    #[clap(short, long, value_enum)]
    from: Option<InputFormat>,
    #[clap(short = 'I', long)]
    inverted: bool,
    #[clap(short, long, value_enum)]
    to: Option<OutputFormat>,
    #[clap(short, long)]
    output: Option<PathBuf>,
    #[clap(name = "FILE", default_value = "-")]
    file: PathBuf,
}

#[derive(Debug, Clone, clap::Args)]
struct DfsArgs {
    #[clap(short, long, value_enum)]
    from: Option<InputFormat>,
    #[clap(short = 'P', long)]
    path: bool,
    #[clap(short = 'S', long)]
    start: Option<String>,
    #[clap(short = 'T', long)]
    tree: bool,
    #[clap(long)]
    max_depth: Option<usize>,
    #[clap(name = "FILE", default_value = "-")]
    file: PathBuf,
}

#[derive(Debug, Clone, clap::Args)]
struct BfsArgs {
    #[clap(short, long, value_enum)]
    from: Option<InputFormat>,
    #[clap(short = 'P', long)]
    path: bool,
    #[clap(short = 'S', long)]
    start: Option<String>,
    #[clap(long)]
    max_depth: Option<usize>,
    #[clap(name = "FILE", default_value = "-")]
    file: PathBuf,
}

#[derive(Debug, Clone, clap::Args)]
struct TsortArgs {
    #[clap(short, long, value_enum)]
    from: Option<InputFormat>,
    #[clap(name = "FILE", default_value = "-")]
    file: PathBuf,
}

fn load_text<R: BufRead>(r: R) -> Result<Graph<String>> {
    itertools::process_results(r.lines(), |iter| {
        let edges = iter.map(|s| {
            let parts: Vec<&str> = s.splitn(2, ' ').collect();
            Edge(parts[0].to_owned(), parts[1].to_owned())
        });
        Graph::from_iter(edges)
    })
    .context("can't load text")
}

fn load_json<R: BufRead>(r: R) -> Result<Graph<String>> {
    let deps: BTreeMap<String, Vec<String>> =
        serde_json::from_reader(r).context("can't load json")?;
    Ok(Graph::from(deps))
}

fn load<R: BufRead>(r: R, format: InputFormat) -> Result<Graph<String>> {
    match format {
        InputFormat::Text => load_text(r),
        InputFormat::Json => load_json(r),
    }
}

fn load_with_path(p: &Path, format: Option<InputFormat>) -> Result<Graph<String>> {
    let format = format
        .as_ref()
        .cloned()
        .unwrap_or_else(|| InputFormat::assume_from_path(p));
    debug!("input format: {:?}", format);
    if p == Path::new("-") {
        let stdin_lock = stdin().lock();
        let r = BufReader::new(stdin_lock);
        load(r, format)
    } else {
        let f = File::open(p)?;
        let r = BufReader::new(f);
        load(r, format)
    }
}

fn dump_text<W: Write>(mut w: W, graph: Graph<String>) -> Result<()> {
    for e in graph.to_edges() {
        w.write_all(e.0.as_bytes())?;
        w.write_all(&[b' '])?;
        w.write_all(e.1.as_bytes())?;
        w.write_all(&[b'\n'])?;
    }
    Ok(())
}

fn dump_dot<W: Write>(mut w: W, graph: Graph<String>) -> Result<()> {
    w.write_all(b"digraph {\n")?;
    for (i, n) in graph.values.iter().enumerate() {
        w.write_all(
            format!(
                "    n{} [label=\"{}\"];\n",
                i,
                n.replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
            )
            .as_bytes(),
        )?;
    }
    w.write_all(b"\n")?;
    for e in graph.to_index_edges() {
        w.write_all(format!("    n{} -> n{};\n", e.0, e.1).as_bytes())?;
    }
    w.write_all(b"}\n")?;
    Ok(())
}

fn dump_json<W: Write>(mut w: W, graph: Graph<String>) -> Result<()> {
    serde_json::to_writer(&mut w, &graph.to_btree_map()).context("can't dump json")?;
    w.write_all(b"\n").context("can't dump json")
}

fn dump<W: Write>(w: W, graph: Graph<String>, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => dump_text(w, graph),
        OutputFormat::Json => dump_json(w, graph),
        OutputFormat::Dot => dump_dot(w, graph),
    }
}

fn dump_with_path<P: AsRef<Path>>(
    p: P,
    graph: Graph<String>,
    format: Option<OutputFormat>,
) -> Result<()> {
    let format = format
        .as_ref()
        .cloned()
        .unwrap_or_else(|| OutputFormat::assume_from_path(p.as_ref()));
    debug!("output format: {:?}", format);
    if p.as_ref() == Path::new("-") {
        let stdout_lock = stdout().lock();
        let w = BufWriter::new(stdout_lock);
        dump(w, graph, format)
    } else {
        let swp = NamedTempFile::new_in(p.as_ref().parent().unwrap())?;
        {
            let f = File::create(&swp)?;
            let w = BufWriter::new(f);
            dump(w, graph, format)?
        }
        swp.persist(p)?;
        Ok(())
    }
}

fn show(_args: &Args, subargs: &ShowArgs) -> Result<()> {
    let graph = load_with_path(&subargs.file, subargs.from.clone())?;
    debug!("{:?}", graph);
    let output = subargs.output.clone().unwrap_or_else(|| "-".into());
    if subargs.inverted {
        dump_with_path(output, graph.invert(), subargs.to.clone())?;
    } else {
        dump_with_path(output, graph, subargs.to.clone())?;
    }
    Ok(())
}

fn dfs(_args: &Args, subargs: &DfsArgs) -> Result<()> {
    let graph = load_with_path(&subargs.file, subargs.from.clone())?;
    let is = match &subargs.start {
        Some(k) => vec![graph.value_to_index[k]],
        None => graph.find_roots(),
    };
    if subargs.path {
        let mut path: Vec<String> = vec![];
        dfs::dfs(&graph, is.as_slice(), |i, t, _f| {
            path.resize(i + 1, "".to_owned());
            path[i] = graph.values[t].clone();
            println!("{}", path.join(" "));
            check_max_depth!(i + 1, subargs.max_depth, DEFAULT_MAX_DEPTH, {
                return false;
            });
            true
        });
    } else {
        dfs::dfs(&graph, is.as_slice(), |i, t, f| {
            if subargs.tree {
                println!("{}* {}", " ".repeat(i * 4), graph.values[t]);
            } else {
                let Some(f) = f else { return true; };
                println!("{} {} {}", graph.values[f], graph.values[t], i);
            }
            check_max_depth!(i + 1, subargs.max_depth, DEFAULT_MAX_DEPTH, {
                return false;
            });
            true
        });
    }
    Ok(())
}

fn bfs(_args: &Args, subargs: &BfsArgs) -> Result<()> {
    let graph = load_with_path(&subargs.file, subargs.from.clone())?;
    let is = match &subargs.start {
        Some(k) => vec![graph.value_to_index[k]],
        None => graph.find_roots(),
    };
    if subargs.path {
        bfs::bfs_path(&graph, is.as_slice(), |path| {
            let path: Vec<&str> = path.iter().map(|i| graph.values[*i].as_str()).collect();
            println!("{}", path.join(" "));
            check_max_depth!(path.len(), subargs.max_depth, DEFAULT_MAX_DEPTH, {
                return false;
            });
            true
        });
    } else {
        bfs::bfs(&graph, is.as_slice(), |i, t, f| {
            let Some(f) = f else { return true; };
            println!("{} {} {}", graph.values[f], graph.values[t], i);
            check_max_depth!(i + 1, subargs.max_depth, DEFAULT_MAX_DEPTH, {
                return false;
            });
            true
        });
    }
    Ok(())
}

fn tsort(_args: &Args, subargs: &TsortArgs) -> Result<()> {
    let graph = load_with_path(&subargs.file, subargs.from.clone())?;
    let result = tsort::tsort(&graph, |t| {
        println!("{}", graph.values[t]);
    });
    if let Err(remaining) = result {
        warn!("contains a loop");
        let remaining: Vec<_> = remaining.iter().copied().sorted().collect();
        for v in remaining.iter() {
            eprintln!("loop: {}", graph.values[*v]);
        }
        for v in remaining.iter() {
            println!("{}", graph.values[*v]);
        }
        exit(1);
    }
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    debug!("{:?}", args);
    match &args.subcommand {
        Subcommand::Show(subargs) => show(&args, subargs),
        Subcommand::Dfs(subargs) => dfs(&args, subargs),
        Subcommand::Bfs(subargs) => bfs(&args, subargs),
        Subcommand::Tsort(subargs) => tsort(&args, subargs),
    }
}
