use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, BufRead, Write},
    process::{exit, Command, Stdio},
};

#[derive(Debug)]
struct Node {
    end: bool,
    children: HashMap<char, Node>,
}

impl Node {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            end: false,
        }
    }
}

fn insert_text(root: &mut Node, text: &str) {
    let mut node = root;
    for ch in text.chars() {
        node = node.children.entry(ch).or_insert(Node::new());
    }
    node.end = true;
}

#[allow(dead_code)]
fn check(root: &Node, text: &str) -> bool {
    let mut node = root;
    for ch in text.chars() {
        if let Some(child) = node.children.get(&ch) {
            node = child;
        } else {
            return false;
        }
    }
    return true;
}

fn dump_dot<T: Write>(file: &mut T, root: &Node, index: &mut u16) -> io::Result<()> {
    let root_index = *index;
    for (item, child) in &root.children {
        *index += 1;
        writeln!(file, "  Node_{} [label=\"{}\"]", index, item)?;
        writeln!(
            file,
            "  Node_{} -> Node_{} [label=\"{}\"]",
            root_index, index, item
        )?;
        dump_dot(file, child, index)?
    }
    Ok(())
}

fn find_prefix<'a>(root: &'a Node, prefix: &str) -> &'a Node {
    let mut node = root;
    for ch in prefix.chars() {
        if let Some(child) = node.children.get(&ch) {
            node = child;
        }
    }
    return node;
}

fn print_autocompletion(root: &Node, buffer: &mut Vec<char>, prefix: &str) -> io::Result<()> {
    if root.end {
        writeln!(
            io::stdout(),
            "{}{}",
            prefix,
            buffer.iter().collect::<String>()
        )?;
        return Ok(());
    }

    for (item, child) in &root.children {
        buffer.push(*item);
        print_autocompletion(child, buffer, prefix)?;
        buffer.pop();
    }
    Ok(())
}

fn usage(mut sink: impl Write) -> io::Result<()> {
    writeln!(sink, "Usage: ./prefix-tree <SUBCOMMAND>")?;
    writeln!(sink, "SUBCOMMANDS")?;
    writeln!(
        sink,
        "    dot               Dump the Trie into a Graphviz dot file."
    )?;
    writeln!(
        sink,
        "    complete <prefix> Suggest prefix autocompletion based on the Trie"
    )?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut root = Node::new();
    let file = File::open("dictionary.txt")?;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        insert_text(&mut root, &line);
    }

    if let Some(subcommand) = env::args().nth(1) {
        match subcommand.as_str() {
            "dot" => {
                let mut dot_file = File::create("trie.dot")?;
                writeln!(&dot_file, "digraph Trie {{")?;
                writeln!(&dot_file, "  Node_{} [label=\"{}\"]", 0, "root")?;
                dump_dot(&mut dot_file, &root, &mut 0)?;
                writeln!(&dot_file, "}}")?;
                let child = Command::new("dot")
                    .arg("-Tsvg")
                    .arg("trie.dot")
                    .stdout(Stdio::piped())
                    .spawn()?;
                let output = child.wait_with_output()?;
                if output.status.success() {
                    let raw_output = String::from_utf8_lossy(output.stdout.as_slice());
                    let mut graph_svg = File::create("trie.svg")?;
                    writeln!(graph_svg, "{}", raw_output)?;
                }
            }
            "complete" => {
                if let Some(prefix) = env::args().nth(2) {
                    let node = find_prefix(&mut root, prefix.as_str());
                    let mut buffer = vec![];
                    print_autocompletion(&node, &mut buffer, &prefix)?;
                }
            }
            _ => {
                writeln!(io::stderr(), "ERROR: no subcommand found.\n")?;
                usage(io::stderr())?;
                exit(1);
            }
        }
    } else {
        usage(io::stderr())?;
        writeln!(io::stderr(), "ERROR: no subcommand is provided")?;
        exit(1);
    }

    Ok(())
}
