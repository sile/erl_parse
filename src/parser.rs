use erl_tokenize::LexicalToken;

use {Result, TokenReader, Parse, Preprocessor, Expect, ParseLeftRecur};

#[derive(Debug)]
pub struct Parser<T> {
    reader: TokenReader<T>,
    transactions: Vec<Vec<LexicalToken>>,
}
impl<T> Parser<T>
where
    T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
{
    pub fn new(reader: TokenReader<T>) -> Self {
        Parser {
            reader,
            transactions: Vec::new(),
        }
    }
    pub fn is_eos(&mut self) -> Result<bool> {
        if let Some(t) = track!(self.reader.try_read_token())? {
            self.reader.unread_token(t);
            Ok(false)
        } else {
            Ok(true)
        }
    }
    pub fn read_token(&mut self) -> Result<LexicalToken> {
        match self.reader.read_token() {
            Err(e) => Err(e),
            Ok(t) => {
                if let Some(tail) = self.transactions.last_mut() {
                    tail.push(t.clone());
                }
                Ok(t)
            }
        }
    }
    pub fn unread_token(&mut self, token: LexicalToken) {
        if let Some(tail) = self.transactions.last_mut() {
            tail.pop();
        }
        self.reader.unread_token(token);
    }
    fn start_transaction(&mut self) {
        self.transactions.push(Vec::new());
    }
    fn commit_transaction(&mut self) {
        let last = self.transactions.pop().unwrap();
        if let Some(tail) = self.transactions.last_mut() {
            tail.extend(last);
        }
    }
    fn abort_transaction(&mut self) {
        let last = self.transactions.pop().unwrap();
        for t in last.into_iter().rev() {
            self.reader.unread_token(t);
        }
    }
    pub fn peek<F, P>(&mut self, f: F) -> Result<P>
    where
        F: FnOnce(&mut Self) -> Result<P>,
    {
        self.start_transaction();
        let result = track!(f(self));
        self.abort_transaction();
        result
    }
    pub fn transaction<F, P>(&mut self, f: F) -> Result<P>
    where
        F: FnOnce(&mut Self) -> Result<P>,
    {
        self.start_transaction();
        let result = track!(f(self));
        if result.is_ok() {
            self.commit_transaction();
        } else {
            self.abort_transaction();
        }
        result
    }
    pub fn parse<P: Parse>(&mut self) -> Result<P> {
        track!(P::parse(self))
    }
    pub fn parse_left_recur<P: ParseLeftRecur>(&mut self, left: P::Left) -> Result<P> {
        track!(P::parse_left_recur(self, left))
    }
    pub fn expect<P: Parse + Expect>(&mut self, expected: &P::Value) -> Result<P> {
        self.transaction(|parser| {
            let actual = track!(parser.parse::<P>())?;
            track!(actual.expect(expected))?;
            Ok(actual)
        })
    }
    pub fn expect_any<P: Parse + Expect>(&mut self, expected: &[&P::Value]) -> Result<P> {
        let actual = track!(self.parse::<P>())?;
        let mut last_error = None;
        for e in expected.iter() {
            if let Err(e) = track!(actual.expect(e)) {
                last_error = Some(e);
            } else {
                last_error = None;
                break;
            }
        }
        if let Some(e) = last_error {
            Err(e)
        } else {
            Ok(actual)
        }
    }
}
