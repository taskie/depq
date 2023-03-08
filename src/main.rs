use std::{
    fs::{rename, File},
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::exit,
};

use anyhow::{Context as _, Result};
use clap::{self, Parser};
use graph::{Edge, Graph};
use itertools::Itertools;
use log::{debug, warn};

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

#[derive(Debug, Clone, clap::Args)]
struct ShowArgs {
    #[clap(short = 'I', long)]
    inverted: bool,
    #[clap(name = "FILE", default_value = "-")]
    file: PathBuf,
}

#[derive(Debug, Clone, clap::Args)]
struct DfsArgs {
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

fn load_text_with_path(p: &Path) -> Result<Graph<String>> {
    if p == Path::new("-") {
        let stdin_lock = stdin().lock();
        let r = BufReader::new(stdin_lock);
        load_text(r)
    } else {
        let f = File::open(p)?;
        let r = BufReader::new(f);
        load_text(r)
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

fn dump_text_with_path<P: AsRef<Path>>(p: P, graph: Graph<String>) -> Result<()> {
    if p.as_ref() == Path::new("-") {
        let stdout_lock = stdout().lock();
        let w = BufWriter::new(stdout_lock);
        dump_text(w, graph)
    } else {
        let mut swp = PathBuf::from(p.as_ref());
        swp.push(".swp");
        {
            let f = File::create(&swp)?;
            let w = BufWriter::new(f);
            dump_text(w, graph)?
        }
        rename(&swp, p)?;
        Ok(())
    }
}

fn show(_args: &Args, subargs: &ShowArgs) -> Result<()> {
    let graph = load_text_with_path(&subargs.file)?;
    debug!("{:?}", graph);
    if subargs.inverted {
        dump_text_with_path("-", graph.invert())?;
    } else {
        dump_text_with_path("-", graph)?;
    }
    Ok(())
}

fn dfs(_args: &Args, subargs: &DfsArgs) -> Result<()> {
    let graph = load_text_with_path(&subargs.file)?;
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
    let graph = load_text_with_path(&subargs.file)?;
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
    let graph = load_text_with_path(&subargs.file)?;
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
