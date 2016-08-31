use rectangles::Rectangle;

#[derive(Debug)]
pub enum TokenId {
	NumericId(u16),
	BoundingBox(Rectangle),
}

#[derive(Debug)]
pub struct RelevantToken {
	pub id: TokenId,    
	document: Option<String>,
    transcription: Option<String>,
	relevance: Option<f32>,
}

impl RelevantToken {
    pub fn new() -> RelevantToken {
        RelevantToken {
            id: TokenId::NumericId(0),
            document: None,
            relevance: None,
            transcription: None,
        }
    }
    pub fn set_tokenid(&mut self, i: TokenId) -> &mut RelevantToken {
        match i {
            TokenId::NumericId(j)            => self.id = TokenId::NumericId(j),
            TokenId::BoundingBox(j)          => self.id = TokenId::BoundingBox(j),
        }
        self
    }
    pub fn set_document(&mut self, d: String) -> &mut RelevantToken {
        self.document = Some(d); self
    }
    pub fn set_transcription(&mut self, d: String) -> &mut RelevantToken {
        self.transcription = Some(d); self
    }    
    pub fn set_relevance(&mut self, r: f32) -> &mut RelevantToken {
        if r <= 0.0 || r > 1.0 { panic!("Invalid value for relevance") }
        self.relevance = Some(r); self
    }
    pub fn get_relevance(&self) -> f32 {
        match self.relevance {
            Some(r) => r,
            None => 1.0, //this acts as the default relevance value
        }
    }
    pub fn print(&self) {
        println!("RelevantToken with id:{:?}, referring document:{:?}, relevance:{:?}, transcription:{:?}", 
            self.id, 
            self.document, 
            self.relevance,
            self.transcription,
        )
    }
}

#[test]
fn test_relevant_token() {
    let mut tok1 = RelevantToken::new();
    tok1.set_tokenid(TokenId::NumericId(34))
        .set_document(String::from("doc0010"))
        .set_relevance(0.8);
    let tok1 = tok1;

    let mut tok2 = RelevantToken::new();
    tok2.set_tokenid(TokenId::BoundingBox(Rectangle::new()))
        .set_relevance(0.8);
    let tok2 = tok2;

    let mut tok3 = RelevantToken::new();
    tok3.set_tokenid(TokenId::NumericId(34))
        .set_document(String::from("doc0010"))
        .set_transcription(String::from("Hello"));
    let tok3 = tok3;
    // Run with cargo test -- --nocapture
    // or else nothing will show up if the test is succesful
    tok1.print();
    tok2.print();
    tok3.print();
}