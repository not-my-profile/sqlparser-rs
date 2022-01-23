// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![warn(clippy::all)]

/// A small command-line app to run the parser.
/// Run with `cargo run --example cli`
use std::fs;

use simple_logger::SimpleLogger;
use sqlparser::dialect::*;
use sqlparser::parser::Parser;

fn main() {
    SimpleLogger::new().init().unwrap();

    let filename = std::env::args().nth(1).expect(
        r#"
No arguments provided!

Usage:
$ cargo run --example cli FILENAME.sql [--dialectname]

To print the parse results as JSON:
$ cargo run --feature json_example --example cli FILENAME.sql [--dialectname]

"#,
    );

    match std::env::args().nth(2).unwrap_or_default().as_ref() {
        "--ansi" => parse::<AnsiDialect>(filename),
        "--postgres" => parse::<PostgreSqlDialect>(filename),
        "--ms" => parse::<MsSqlDialect>(filename),
        "--mysql" => parse::<MySqlDialect>(filename),
        "--snowflake" => parse::<SnowflakeDialect>(filename),
        "--hive" => parse::<HiveDialect>(filename),
        "--generic" | "" => parse::<GenericDialect>(filename),
        s => panic!("Unexpected parameter: {}", s),
    };
}

fn parse<D: Dialect>(filename: String) {
    println!(
        "Parsing from file '{}' using {:?}",
        &filename,
        std::any::type_name::<D>()
    );
    let contents = fs::read_to_string(&filename)
        .unwrap_or_else(|_| panic!("Unable to read the file {}", &filename));
    let without_bom = if contents.chars().next().unwrap() as u64 != 0xfeff {
        contents.as_str()
    } else {
        let mut chars = contents.chars();
        chars.next();
        chars.as_str()
    };
    let parse_result = Parser::<D>::parse_sql(without_bom);
    match parse_result {
        Ok(statements) => {
            println!(
                "Round-trip:\n'{}'",
                statements
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            if cfg!(feature = "json_example") {
                #[cfg(feature = "json_example")]
                {
                    let serialized = serde_json::to_string_pretty(&statements).unwrap();
                    println!("Serialized as JSON:\n{}", serialized);
                }
            } else {
                println!("Parse results:\n{:#?}", statements);
            }

            std::process::exit(0);
        }
        Err(e) => {
            println!("Error during parsing: {:?}", e);
            std::process::exit(1);
        }
    }
}
