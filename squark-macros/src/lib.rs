#![feature(proc_macro, proc_macro_derive)]

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate proc_macro;

use proc_macro::{quote, TokenStream, TokenNode, Literal, Term};
use pest::Parser;
use pest::iterators::{Pairs, Pair};
use parser::{Parser as ViewParser, Rule};
use std::iter::FromIterator;
use std::str::FromStr;

mod parser {
    #[derive(Parser)]
    #[grammar = "view.pest"]
    pub struct Parser;
}

fn get_token_stream(mut tag_pairs: Pairs<Rule>) -> TokenStream {
    let name = tag_pairs.next().expect("name").as_str();
    let _name = TokenNode::Literal(Literal::string(name));

    let mut attributes = vec![];
    let mut handlers = vec![];

    let vec: Vec<Pair<Rule>> = tag_pairs.next().expect("attributes").into_inner().collect();
    for i in 0..(vec.len() / 2) {
        let j = i * 2;
        let k = &vec[j].as_str();
        let v = &vec[j+1];

        let _v = match v.as_rule() {
            Rule::embedded => {
                TokenStream::from_str(v.as_str()).unwrap()
            },
            Rule::string => {
                let _v = TokenNode::Literal(Literal::string(v.as_str()));
                quote! { _squark::string($_v.to_string()) }
            },
            Rule::bool => {
                let _v = TokenNode::Term(Term::intern(v.as_str()));
                quote! { _squark::bool($_v) }
            },
            _ => unreachable!(),
        };

        if k.starts_with("on") {
            let (_, k) = k.split_at(2);
            let _k = TokenNode::Literal(Literal::string(k));
            handlers.push(quote! {
                ($_k.to_string(), $_v),
            });
            continue;
        }

        let _k = TokenNode::Literal(Literal::string(k));
        attributes.push(quote! {
            ($_k.to_string(), $_v),
        });
    }
    let _attributes = TokenStream::from_iter(attributes);
    let _handlers = TokenStream::from_iter(handlers);


    let mut children = vec![];
    if let Some(children_pair) = tag_pairs.next() {
        for p in children_pair.into_inner() {
            let token = match p.as_rule() {
                Rule::tag => {
                    let _tag = get_token_stream(p.into_inner());
                    quote! {
                        $_tag,
                    }
                },
                Rule::text => {
                    let _text = TokenNode::Literal(Literal::string(p.as_str()));
                    quote! {
                        _squark::text($_text.to_string()),
                    }
                },
                Rule::embedded => {
                    let _embedded = TokenStream::from_str(p.as_str()).unwrap();
                    quote! {
                        $_embedded,
                    }
                },
                _ => unreachable!(),
            };
            children.push(token);
        }
    }
    let _children = TokenStream::from_iter(children);

    quote! {
        _squark::view(
            $_name.to_string(),
            vec![
                $_attributes
            ],
            vec![
                $_handlers
            ],
            vec![
                $_children
            ]
        )
    }
}

#[proc_macro]
pub fn view(arg: TokenStream) -> TokenStream {
    let s = arg.to_string();
    let mut pairs = ViewParser::parse(Rule::view, &s).unwrap();
    let _token = get_token_stream(pairs.next().unwrap().into_inner());

    quote! {
        {
            extern crate squark as _squark;
            $_token
        }
    }
}