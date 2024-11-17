use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_until1},
    character::complete::{char, line_ending, not_line_ending},
    combinator::{eof, map, recognize},
    error::ParseError,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use regex::Regex;
use std::fs;

fn to_es_module(js: &str) -> String {
    let (_, result) = preceded(
        tag("(function(scope){"),
        declaration_parser,
        //tag("}(this))"),
    )(js)
    .expect("Parser succeeds");
    result
}

// Parser to recognize and skip _Platform_export functions
fn skip_platform_export(input: &str) -> IResult<&str, &str> {
    preceded(tag("function _Platform_export"), skip_function_block)(input)
}

// Parser to recognize and skip _Platform_mergeExports functions
fn skip_platform_merge_exports(input: &str) -> IResult<&str, &str> {
    preceded(tag("function _Platform_mergeExports"), skip_function_block)(input)
}

// Generic parser for anything that is not an export function
fn parse_other(input: &str) -> IResult<&str, &str> {
    not_line_ending(input)
}

// Main parser that filters function content
fn declaration_parser(input: &str) -> IResult<&str, String> {
    let mut done = false;
    let mut leftover = input;
    let mut output = String::new();
    let mut count = 3;
    while !done && count > 0 {
        take_until1("function")(input)?;

        count -= 1;
        println!("done {done} {output}");
        let (new_leftover, (partial_output, new_done)) = alt((
            skip_platform_export.map(|_| ("", false)),
            skip_platform_merge_exports.map(|_| ("", false)),
            recognize(preceded(tag("function"), skip_function_block)).map(|out| (out, false)),
            take_until("function").map(|out| (out, false)),
            eof.map(|_| ("", true)),
        ))(leftover)?;
        println!("new leftover: {}", &new_leftover[0..100]);
        leftover = new_leftover;
        done = new_done;
        output += partial_output;
    }
    Ok((leftover, output))
}

fn skip_function_block(input: &str) -> IResult<&str, &str> {
    let mut depth = 0;

    let (mut input, _) = tag("{")(input)?;
    depth += 1;

    while depth > 0 {
        let (rest, change) = alt((
            map(char('{'), |_| 1),
            map(char('}'), |_| -1),
            map(is_not("{}"), |_| 0),
        ))(input)?;
        depth += change;
        input = rest;
    }

    Ok((input, ""))
}

#[test]
fn run() {
    let file = fs::read_to_string("./binding.js").unwrap();
    let es = to_es_module(&file);
    println!("{es}");
    //fs::write("./binding2.js", es).unwrap();
}
