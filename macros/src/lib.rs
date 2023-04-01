use std::{str::FromStr, collections::BTreeSet};

use proc_macro::TokenStream;

#[proc_macro]
pub fn levels(tokens: TokenStream) -> TokenStream {
    let declaration_of_levels = tokens.to_string();
    let mut levels = BTreeSet::new();
    let mut output = String::from("Levels{data:[");
    for declaration in declaration_of_levels.split(',') {
        let declaration = declaration.trim_start().trim_end();
        if declaration.is_empty() {
            continue;
        }
        let Some((severity, name)) = declaration.split_once("=>") else {
            panic!("Syntax: <severity> => <name>");
        };
        let Ok(severity_parsed) = severity.trim_start().trim_end().parse::<usize>() else {
            panic!("Use non-negative integer as severity!");
        };
        if !levels.insert(severity_parsed) {
            panic!("All severities need to be different");
        }
        output.push('(');
        output.push_str(name);
        output.push(',');
        output.push_str(severity);
        output.push_str("),");
    }
    output.push_str("]}");
    TokenStream::from_str(&output).unwrap()
}
