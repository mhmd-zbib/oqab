use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("oqab")
        .version("0.1.0")
        .author("mhmd-zbib")
        .about("Optimized Query Analyzer for Big Data")
        .arg(
            Arg::new("input")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Sets the input file or directory to search")
                .required(true),
        )
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("TARGET")
                .help("Sets the search target string")
                .required(true),
        )
        .arg(
            Arg::new("max-depth")
                .short('d')
                .long("max-depth")
                .value_name("DEPTH")
                .help("Sets the maximum depth for directory traversal (default: 3)")
                .default_value("3"),
        )
        .arg(
            Arg::new("threads")
                .short('j')
                .long("threads")
                .value_name("THREADS")
                .help("Number of threads to use (default: number of logical cores)")
                .default_value("0"),
        )
}
