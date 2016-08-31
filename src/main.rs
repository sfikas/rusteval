#![allow(dead_code)]
mod rectangles;
mod tokens;
extern crate regex;
extern crate getopts;
extern crate rand;

use std::env;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::collections::HashMap;
use regex::Regex;
use getopts::Options;

use rectangles::Rectangle;
use tokens::RelevantToken;
use tokens::TokenId;

enum FileType {
	ResultsFile,
	RelevantTokensFile,
}

#[derive(Debug)]
enum BenchmarkResult {
	Scalar(f32),
	Vector(Vec<f32>),
	None,
}

struct RetrievalData {
	relevant_tokens_filename: 	Option<String>,
	relevant_tokens: 			Option<HashMap<String, Vec<RelevantToken>>>,
	result_tokens_filename:		Option<String>,	
	result_tokens: 				Option<HashMap<String, Vec<RelevantToken>>>,
	benchmark_results:			HashMap<String, HashMap<String, BenchmarkResult>>,
}

impl RetrievalData {
	fn new() -> RetrievalData {
		RetrievalData { 
			relevant_tokens_filename: None,
			relevant_tokens: None,
			result_tokens_filename: None,			
			result_tokens: None,
			benchmark_results: HashMap::new(),
		}
	}
	fn set_relevants_filename(&mut self, fname: String) { self.relevant_tokens_filename = Some(fname); }
	fn set_results_filename(&mut self, fname: String) { self.result_tokens_filename = Some(fname); }	
}

trait MetricPrecisionAtX {
	fn precision_at_x(&self, queryname: &String, results: &Vec<RelevantToken>, x: usize) -> BenchmarkResult;
	fn precision_at_5(&self, queryname: &String, results: &Vec<RelevantToken>) -> BenchmarkResult {
		self.precision_at_x(queryname, results, 5)
	}	
	fn precision_at_10(&self, queryname: &String, results: &Vec<RelevantToken>) -> BenchmarkResult {
		self.precision_at_x(queryname, results, 10)
	}		
}

trait MetricMAP {
	fn num_recall_points(&self) -> usize { 11 }
	fn average_precision(&self, queryname: &String, results: &Vec<RelevantToken>) -> BenchmarkResult;
}

trait Benchmark : MetricPrecisionAtX + MetricMAP {
	fn tokens_match(&self, a: &RelevantToken, b: &RelevantToken) -> bool;	
	fn is_this_a_hit(&self, queryname: &String, token: &RelevantToken) -> bool;
	fn store_all_numerical_results(&mut self, queryname: &String, hitlist: &Vec<RelevantToken>);
	fn computed_benchmarks(&self) -> Vec<String>;	
	fn print_all_benchmarks(&self);
	fn compute_average_benchmark(&self, benchmark: &String) -> BenchmarkResult;
}

trait ParserTrecEval : Benchmark {
}

trait ParserXmlICFHR14 : Benchmark {
	fn parse_tokenstring_fast(&self, tokstr: String) -> RelevantToken;
	fn parse_tokenstring(&self, tokstr: String) -> RelevantToken;
	fn parse_file(&mut self, ft: FileType);
}

impl Benchmark for RetrievalData {
	fn tokens_match(&self, a: &RelevantToken, b: &RelevantToken) -> bool {
		let a_box = match a.id {
			TokenId::BoundingBox(ref r) => r,
			_ => panic!("Not implemented yet!"),
		};
		let b_box = match b.id {
			TokenId::BoundingBox(ref r) => r,
			_ => panic!("Not implemented yet!"),
		};
		//TODO: This should be done as a Rectangle operator trait
		a_box.min.get_x() == b_box.min.get_x() && 
		a_box.min.get_y() == b_box.min.get_y() &&
		a_box.width() == b_box.width() &&
		a_box.height() == b_box.height()
	}
	fn is_this_a_hit(&self, queryname: &String, token: &RelevantToken) -> bool {
		match self.relevant_tokens {
			Some(ref i) => i.get(queryname).unwrap(),
			None => panic!(format!("Can't find relevant tokens list for query {}. Did you load a relevance file?", queryname)),
		}.into_iter().any( |pred| self.tokens_match(pred, token))
	}
	fn store_all_numerical_results(&mut self, queryname: &String, hitlist: &Vec<RelevantToken>) {
		let mut res = HashMap::new();
		res.insert(
			String::from("precAt5"),
			self.precision_at_5(queryname, hitlist),
		);
		res.insert(
			String::from("precAt10"),
			self.precision_at_10(queryname, hitlist),
		);
		res.insert(
			String::from("ap"),
			self.average_precision(queryname, hitlist),
		);		
		self.benchmark_results.insert(queryname.clone(), res);
	}
	fn computed_benchmarks(&self) -> Vec<String> {
		self.benchmark_results.values().nth(0).unwrap().keys().cloned().collect()
	}
	fn print_all_benchmarks(&self) {
		let ref res = self.benchmark_results;
		let benchmarks = self.computed_benchmarks();
		print!("\t\t");
		for b in benchmarks.clone().into_iter() {
			print!("{:width$}", b, width = 16)
		}
		println!("");
		println!("=======================================================================");
		for (queryname, v) in res.iter() {
			print!("{}", queryname);
			for b in benchmarks.clone().into_iter() {
				let results = v.get(&b[..]).unwrap(); 
				match results {
					&BenchmarkResult::Scalar(f) => print!("\t\t{:1.5}", f),
					&BenchmarkResult::Vector(_) => panic!("Don't know how to print a vector"),
					&BenchmarkResult::None => {},		
				}
			}
			println!("");
		}
		println!("------------------------------------------------------------------------");
		print!("MEAN:\t\t");
		for b in benchmarks.clone().into_iter() {
			print!("{:width$}", b, width = 16)
		}
		println!("");
		println!("=======================================================================");
		print!("\t\t");				
		for b in benchmarks.clone().into_iter() {
			if let BenchmarkResult::Scalar(score) = self.compute_average_benchmark(&b) {
				print!("{:1.5}\t\t", score);
			}
		}
		println!("");
	}
	fn compute_average_benchmark(&self, benchmark: &String) -> BenchmarkResult {
		let res = self.benchmark_results.values();
		let mut acc = 0.0;
		for v in res.clone().into_iter() {
			let results = v.get(benchmark).unwrap(); 
			match results {
				&BenchmarkResult::Scalar(f) => acc += f,
				&BenchmarkResult::Vector(_) => panic!("Don't know how to print a vector"),
				&BenchmarkResult::None => panic!("A query has no computed value"),		
			};
		}
		let c: f32 = res.count() as f32;
		BenchmarkResult::Scalar(acc / c)
	}	
}

impl MetricMAP for RetrievalData {
	fn average_precision(&self, queryname: &String, results: &Vec<RelevantToken>) -> BenchmarkResult {
		let hitcount: Vec<f32> = results.iter()
			.map( |tok| if self.is_this_a_hit(&queryname, tok) { 1.0 } else { 0.0 })
			.collect();		
		let num_relevants = match self.relevant_tokens {
			Some(ref i) => i.get(queryname).unwrap(),
			None => panic!(format!("MetricMAP::average_precision: Can't find relevant tokens list for query {}. Did you load a relevance file?", queryname)),
		}.into_iter().count() as f32;
		let hitsum: Vec<f32> = hitcount.clone().into_iter()
			.scan(0.0, |state, x| {
				*state = *state + x;
				Some(*state)
			})
			.enumerate()
			.map( |(i, x)| x/(i as f32 + 1.0) )
			.collect();			
		BenchmarkResult::Scalar(hitcount
			.iter()
			.zip(hitsum.iter())
			.fold(0.0, |acc, (a,b)| acc + a*b ) / num_relevants
		)
	}	
}

impl MetricPrecisionAtX for RetrievalData {
	fn precision_at_x(&self, queryname: &String, results: &Vec<RelevantToken>, x: usize) -> BenchmarkResult {
		let topx = &results[0..x];
		let hitcount = topx.into_iter()
			.map( |&ref tok| if self.is_this_a_hit(&queryname, tok) { 1.0 } else { 0.0 })
			.fold(0.0, |acc, i| acc + i);
		let mut denominator = x as f32;
		let num_relevants = match self.relevant_tokens {
			Some(ref i) => i.get(queryname).unwrap(),
			None => panic!(format!("MetricPrecisionAtX::precision_at_x: Can't find relevant tokens list for query {}. Did you load a relevance file?", queryname)),
		}.into_iter().count() as f32;
		if num_relevants < denominator { denominator = num_relevants }
		BenchmarkResult::Scalar(hitcount / denominator)
	}
}

impl ParserXmlICFHR14 for RetrievalData {
	fn parse_tokenstring_fast(&self, tokstr: String) -> RelevantToken {
		let mut next_token = 0;
		let mut invalue = 0;
		let mut endpoints = [[0usize; 2]; 7];
		
		let mut tok = RelevantToken::new();
		for (n, c) in tokstr.chars().enumerate() {
			match c {
				'"' => {
					endpoints[next_token][invalue] = n;
					invalue = 1 - invalue;
					if invalue == 0 { next_token += 1; }
				},
				_ => {},
			}
		}
		let document = String::from(&tokstr[endpoints[0][0]+1..endpoints[0][1]]);
		let x = String::from(&tokstr[endpoints[1][0]+1..endpoints[1][1]]).parse().unwrap();
		let y = String::from(&tokstr[endpoints[2][0]+1..endpoints[2][1]]).parse().unwrap();
		let w = String::from(&tokstr[endpoints[3][0]+1..endpoints[3][1]]).parse().unwrap();
		let h = String::from(&tokstr[endpoints[4][0]+1..endpoints[4][1]]).parse().unwrap();
		if endpoints[5][1] > 0 {
			let imp_char_5 = tokstr.chars().nth(endpoints[5][0]-2).unwrap();
			if imp_char_5 == 'e' // The last character of 'Relevance'
				{ 
					let relv = String::from(&tokstr[endpoints[5][0]+1..endpoints[5][1]]).parse().unwrap(); 
					tok.set_relevance(relv);
				}
			else if imp_char_5 == 't' // The last character of 'Text'
				{ 
					let transcription = String::from(&tokstr[endpoints[5][0]+1..endpoints[5][1]]); 
					tok.set_transcription(transcription);
				}
			else
				{ panic!("Unidentified token field"); }
			if endpoints[6][1] > 0 {
				let imp_char_6 = tokstr.chars().nth(endpoints[6][0]-2).unwrap();				
				if imp_char_6 == 'e' // The last character of 'Relevance'
					{ 
						let relv = String::from(&tokstr[endpoints[6][0]+1..endpoints[6][1]]).parse().unwrap(); 
						tok.set_relevance(relv);
					}
				else if imp_char_6 == 't' // The last character of 'Text'
					{ 
						let transcription = String::from(&tokstr[endpoints[6][0]+1..endpoints[6][1]]); 
						tok.set_transcription(transcription);
					}
				else
					{ panic!("Unidentified token field"); }		
			}
		}
		let mut rect = Rectangle::new();
		rect.set_min(x, y).set_size(w, h);
		tok.set_tokenid(TokenId::BoundingBox(rect));
		tok.set_document(document);
		tok
	}
	fn parse_tokenstring(&self, tokstr: String) -> RelevantToken {		
		let mut token_traits_hash = HashMap::new();
		token_traits_hash.insert("bbox", 		r#"x="(\d+)" y="(\d+)" width="(\d+)" height="(\d+)""#);
		token_traits_hash.insert("id", 	 		r#"id="(\d+)"#);
		token_traits_hash.insert("docname", 	r#"word document="(.*?)""#);
		token_traits_hash.insert("relevance",	r#"Relevance="([\d\.]+)""#);
		let token_traits_hash = token_traits_hash;
		
		let mut tok = RelevantToken::new();			
		for (&k, &val) in &token_traits_hash {
			let re = Regex::new(val).unwrap();
			for l in re.captures_iter(&tokstr[..]) {
				match k {
					"bbox"		=> {
						let mut rect = Rectangle::new();
						let x: u32 = String::from(l.at(1).unwrap()).parse().expect("oops");
						let y: u32 = String::from(l.at(2).unwrap()).parse().expect("oops");
						let w: u32 = String::from(l.at(3).unwrap()).parse().expect("oops");
						let h: u32 = String::from(l.at(4).unwrap()).parse().expect("oops");
						rect.set_min(x, y).set_size(w, h);
						tok.set_tokenid(TokenId::BoundingBox(rect));
					},
					"id"			 => panic!("This isn't implemented yet"),
					"docname"		 => { tok.set_document(String::from(l.at(1).unwrap())); },
					"relevance" 	 => {
						let relv: f32 = String::from(l.at(1).unwrap()).parse().expect("oops");
						tok.set_relevance(relv);
					},
					_				 => panic!("Don't know how to handle this token trait."),
				}
			}
		}
		tok
	}

	fn parse_file(&mut self, ft: FileType) {
		enum ParsingState {
			WaitingNextQuery,
			InQueryBlock(String),
		};
		let (filename, re_querystarts, re_queryends) = match ft {
			FileType::RelevantTokensFile => ( 
				self.relevant_tokens_filename.clone(),				
				Regex::new(r#"<GTRel queryid="([\s\S]*?)">"#).unwrap(),
				Regex::new(r#"</GTRel>"#).unwrap(),

				),
			FileType::ResultsFile => (
				self.result_tokens_filename.clone(),				
				Regex::new(r#"<Rel queryid="([\s\S]*?)">"#).unwrap(),
				Regex::new(r#"</Rel>"#).unwrap(),				
			),
		};
		let filename = match filename { //see http://stackoverflow.com/a/28035122/5615276
			Some(ref f) => f,
			None => panic!("oops"),
		};
		let f = File::open(filename).expect("Input xml could not be read.");
		let f = BufReader::new(f);

		let mut relevant_tokens: Vec<RelevantToken> = Vec::new();
		let mut res = HashMap::new();
		
		let mut state = ParsingState::WaitingNextQuery;		
		let mut re_state_must_change = re_querystarts.clone();
		
		for buffer in f.lines() {
			let current_line = buffer.unwrap();
			match re_state_must_change.captures_iter(&current_line[..]).next() {
				Some(captured_queryname) => match state {
					ParsingState::WaitingNextQuery => {
						state = ParsingState::InQueryBlock(String::from(captured_queryname.at(1).unwrap()));
						re_state_must_change = re_queryends.clone();
					},
					ParsingState::InQueryBlock(query_name) => {
						match ft {
							FileType::ResultsFile => {
								self.store_all_numerical_results(&query_name, &relevant_tokens); 
							},							
							_ => {},
						};
						res.insert(query_name.clone(), relevant_tokens);
						relevant_tokens = Vec::new();
						state = ParsingState::WaitingNextQuery;
						re_state_must_change = re_querystarts.clone();
					},
				},
				None => match state {
					ParsingState::WaitingNextQuery => {},
					ParsingState::InQueryBlock(_) => {
						relevant_tokens.push(self.parse_tokenstring_fast(String::from(current_line.clone())));						
					},
				},
			};
		}
		match ft {
			FileType::RelevantTokensFile => self.relevant_tokens = Some(res),
			FileType::ResultsFile => self.result_tokens = Some(res),
		};
	}
}

fn load_fixtures(load_results: bool) -> RetrievalData {
	let mut f = RetrievalData::new();
	f.set_relevants_filename(String::from("fixtures/GroundTruthRelevanceJudgementsSample.xml"));
	f.parse_file(FileType::RelevantTokensFile);	
	if load_results {
		f.set_results_filename(String::from("fixtures/WordSpottingResultsSample.xml"));
		f.parse_file(FileType::ResultsFile);
	}
	f
}

fn load_fixtures_bentham(load_results: bool) -> RetrievalData {
	let mut f = RetrievalData::new();
	f.set_relevants_filename(String::from("fixtures/TRACK_I_Bentham_ICFHR2014.RelevanceJudgements.xml"));
	f.parse_file(FileType::RelevantTokensFile);	
	if load_results {
		File::open(String::from("fixtures/G1_TRACK_I_Bentham.xml"))
		.expect("You have to unzip the fixture XML first; please follow the instructions found in README.md");
		f.set_results_filename(String::from("fixtures/G1_TRACK_I_Bentham.xml"));
		f.parse_file(FileType::ResultsFile);
	}
	f
}

#[test]
fn test_parserelevants_type_icfhr14xml_checkquerynames() {
	let f = load_fixtures(false);
	let res = f.relevant_tokens.unwrap();
	let num_queries = res.iter().count();
	assert!(res.contains_key("sb0000"));
	assert!(res.contains_key("sb0001"));
	assert_eq!(num_queries, 2);
}

#[test]
fn test_parserelevants_type_icfhr14xml_checksums_1() {
	let f = load_fixtures(false);
	let res = f.relevant_tokens.unwrap();
	assert!(res.values().all( |relevants| relevants.len() == 9 ));
}

#[test]
fn test_parserelevants_type_icfhr14xml_checksums_x() {
	let f = load_fixtures(false);
	let res = f.relevant_tokens.unwrap();
	for token in res.get("sb0000").unwrap().iter() {
		match token.id {
			//Regarding 'ref', see http://stackoverflow.com/a/28159407/5615276
			// and http://rustbyexample.com/scope/borrow/ref.html
			TokenId::BoundingBox(ref bb) => println!("{:?}", bb.min.get_x()),
			TokenId::NumericId(_) 	 => assert!(false),
		}
	}
	assert_eq!(res.get("sb0000").unwrap().iter().fold(0, |acc, token|
		match token.id {
			TokenId::BoundingBox(ref bb) => acc + bb.min.get_x(),
			TokenId::NumericId(_) 	 => acc,
		}
	), 6435);
}

#[test]
fn test_parserelevants_type_icfhr14xml_checksums_y() {
	let f = load_fixtures(false);
	let res = f.relevant_tokens.unwrap();
	assert_eq!(res.get("sb0001").unwrap().iter().fold(0, |acc, token|
		match token.id {
			TokenId::BoundingBox(ref bb) => acc + bb.min.get_y(),
			TokenId::NumericId(_) 	 => acc,
		}
	), 10921);
}

#[test]
fn test_parserelevants_type_icfhr14xml_checksums_width() {
	let f = load_fixtures(false);
	let res = f.relevant_tokens.unwrap();
	assert_eq!(res.get("sb0001").unwrap().iter().fold(0, |acc, token|
		match token.id {
			TokenId::BoundingBox(ref bb) => acc + bb.width(),
			TokenId::NumericId(_) 	 => acc,
		}
	), 2175);
}

#[test]
fn test_parserelevants_type_icfhr14xml_checksums_height() {
	let f = load_fixtures(false);
	let res = f.relevant_tokens.unwrap();
	assert_eq!(res.get("sb0001").unwrap().iter().fold(0, |acc, token|
		match token.id {
			TokenId::BoundingBox(ref bb) => acc + bb.height(),
			TokenId::NumericId(_) 	 => acc,
		}
	), 987);
}

#[test]
fn test_parserelevants_type_icfhr14xml_checksums_relevance_0() {
	let f = load_fixtures(false);
	let res = f.relevant_tokens.unwrap();
	assert_eq!(res.get("sb0000").unwrap().iter().fold(0.0, |acc, token|
		acc + token.get_relevance()
	), 8.0);
}

#[test]
fn test_parserelevants_type_icfhr14xml_checksums_relevance_1() {
	let f = load_fixtures(false);
	let res = f.relevant_tokens.unwrap();
	assert_eq!(res.get("sb0001").unwrap().iter().fold(0.0, |acc, token|
		acc + token.get_relevance()
	), 8.0); 
	//NOTE: there is a syntax error on the sample file ("elevance" instead of "Relevance")
	// This version of the parser will read relevance despite the syntax error;
	// a previous version of this souce didn't though, so the '8.0' on the assert had to be '7.0'.
}

#[test]
fn test_parseresults_type_icfhr14xml_checksums_width() {
	let f = load_fixtures(true);
	let res = f.result_tokens.unwrap();
	let ref token = res.get("sb0001").unwrap()[0]; 
	match token.id {
		TokenId::BoundingBox(ref bb) => assert_eq!(bb.width(), 180),
		TokenId::NumericId(_) 	 => assert!(false),
	};
	let ref token = res.get("sb0001").unwrap().last().unwrap(); 
	match token.id {
		TokenId::BoundingBox(ref bb) => assert_eq!(bb.width(), 278),
		TokenId::NumericId(_) 	 => assert!(false),
	};
}

#[test]
fn test_precision_at_x() {
	let f = load_fixtures(true);
	let res = f.benchmark_results;
	let q0 = res.get("sb0000").unwrap();
	let q1 = res.get("sb0001").unwrap();
	match q0.get("precAt10").unwrap() { 
		&BenchmarkResult::Scalar(x) => assert!((x - 0.55555).abs() < 0.001),
		_ => assert!(false),
	};
	match q1.get("precAt10").unwrap() { 
		&BenchmarkResult::Scalar(x) => assert!((x - 0.44444).abs() < 0.001),
		_ => assert!(false),
	};	
	match q0.get("precAt5").unwrap() { 
		&BenchmarkResult::Scalar(x) => assert_eq!(x, 1.0),
		_ => assert!(false),
	};
	match q1.get("precAt5").unwrap() { 
		&BenchmarkResult::Scalar(x) => assert_eq!(x, 0.8),
		_ => assert!(false),
	};		
}

#[test]
fn test_bentham_precision() {
	let f = load_fixtures_bentham(true);
	let benchmarks = ["precAt10", "precAt5", "ap"];
	// These results coincide with the ones reported at the ICFHR'14 competition, [Pratikakis et al. 2014]
	let should_compute = [0.60267615, 0.73812526, 0.5240239];	
	for (i, &b) in benchmarks.clone().into_iter().enumerate() {
		if let BenchmarkResult::Scalar(score) = f.compute_average_benchmark(&String::from(b)) {
			println!("{} should have been {}, computed is {}", b, should_compute[i], score);
			assert!((should_compute[i] - score).abs() < 0.001); 
		}
	}
}


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} RELEVANCE_FILE RESULT_FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {	
	let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    //opts.optopt("s", "result", "set result file", "NAME");
    //opts.optopt("l", "relevance", "set relevance file", "NAME");	
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    //let input = 
	if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

	let relevance_file = String::from(args[1].clone());
	let result_file = String::from(args[2].clone());
	let mut f = RetrievalData::new();
	//f.set_relevants_filename(String::from("/tmp/gt.xml"));
	//f.set_relevants_filename(String::from("fixtures/GroundTruthRelevanceJudgementsSample.xml"));
	f.set_relevants_filename(relevance_file);
	f.parse_file(FileType::RelevantTokensFile);
	//println!("{:?}", f.relevant_tokens);

	//f.set_results_filename(String::from("/tmp/res.xml"));
	//f.set_results_filename(String::from("fixtures/WordSpottingResultsSample.xml"));
	f.set_results_filename(result_file);	
	f.parse_file(FileType::ResultsFile);
	//println!("{:?}", f.result_tokens);
	//println!("{:?}", f.benchmark_results);
	f.print_all_benchmarks();	
}