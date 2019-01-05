use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use keikaku::lexer::Source;
use keikaku::parser::Ast;

fn main() {
    fn abort(msg: &str) -> ! {
        eprintln!("{}", msg);
        std::process::exit(1);
    }

    let filename = std::env::args().nth(1).unwrap_or_else(|| {
        abort("Source file not specified");
    });
    let file = File::open(Path::new(&filename)).unwrap_or_else(|e| {
        abort(&format!("Could not open source file: {}", e));
    });

    let mut source = String::new();
    BufReader::new(file)
        .read_to_string(&mut source)
        .unwrap_or_else(|e| abort(&format!("Failed to read source file: {}", e)));

    let mut chars = source.chars();
    let lexer = Source::new(filename, &mut chars);
    let ast = Ast::parse(&mut lexer.tokenize().peekable())
        .unwrap_or_else(|e| abort(&format!("Parse error: {:?}", e)));
    println!("{:?}", ast);
}
