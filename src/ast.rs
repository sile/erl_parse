use num::bigint::BigUint;

#[derive(Debug, Clone, PartialEq)]
pub struct Ast;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Var(Var),
    Tuple(Tuple<Expr>),
    Op(Op<Expr>),
}
impl From<Literal> for Expr {
    fn from(f: Literal) -> Self {
        Expr::Literal(f)
    }
}
impl From<u64> for Expr {
    fn from(f: u64) -> Self {
        Expr::Literal(From::from(f))
    }
}
impl From<char> for Expr {
    fn from(f: char) -> Self {
        Expr::Literal(Literal::Char(f))
    }
}
impl From<Op<Expr>> for Expr {
    fn from(f: Op<Expr>) -> Self {
        Expr::Op(f)
    }
}
impl From<Tuple<Expr>> for Expr {
    fn from(f: Tuple<Expr>) -> Self {
        Expr::Tuple(f)
    }
}
impl From<Var> for Expr {
    fn from(f: Var) -> Self {
        Expr::Var(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple<T>(pub Vec<T>);
impl<T, A, B, C> From<(A, B, C)> for Tuple<T>
    where T: From<A> + From<B> + From<C>
{
    fn from(f: (A, B, C)) -> Self {
        Tuple(vec![From::from(f.0), From::from(f.1), From::from(f.2)])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Float(f64),
    Integer(BigUint),
    Atom(String),
    Char(char),
    String(String),
}
impl From<u64> for Literal {
    fn from(f: u64) -> Self {
        Literal::Integer(BigUint::from(f))
    }
}
impl From<String> for Literal {
    fn from(f: String) -> Self {
        Literal::Atom(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op<T> {
    And(And<T>),
    Andalso(Andalso<T>),
    Band(Band<T>),
    Bor(Bor<T>),
    Bsl(Bsl<T>),
    Bsr(Bsr<T>),
    Bxor(Bxor<T>),
    Or(Or<T>),
    Orelse(Orelse<T>),
    Xor(Xor<T>),
    Match(Match<T>),
    DivFloat(DivFloat<T>),
    Div(Div<T>),
    Rem(Rem<T>),
    Sub(Sub<T>),
    ListSub(ListSub<T>),
    Add(Add<T>),
    ListAdd(ListAdd<T>),
    Mul(Mul<T>),
    Eq(Eq<T>),
    ExactEq(ExactEq<T>),
    NotEq(NotEq<T>),
    ExactNotEq(ExactNotEq<T>),
    Greater(Greater<T>),
    GreaterEq(GreaterEq<T>),
    Less(Less<T>),
    LessEq(LessEq<T>),
}
macro_rules! define_binary_op {
    ($op:ident) => { define_binary_op!($op, T, T); };
    ($op:ident, $left:ident) => { define_binary_op!($op, $left, T); };
    ($op:ident, $left:ident, $right:ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $op<T> {
            pub left: Box<$left>,
            pub right: Box<$right>,
        }
        impl<T> $op <T> {
            pub fn new<L, R>(left: L, right: R) -> Self
                where $left: From<L>, $right: From<R> {
                $op {left: Box::new(From::from(left)), right: Box::new(From::from(right))}
            }
        }
        impl<T> From<$op<T>> for Op<T> {
            fn from(f: $op<T>) -> Self {
                Op::$op(f)
            }
        }
        impl From<$op<Expr>> for Expr {
            fn from(f: $op<Expr>) -> Self {
                Expr::from(Op::from(f))
            }
        }
    }
}
define_binary_op!(And);
define_binary_op!(Andalso);
define_binary_op!(Band);
define_binary_op!(Bor);
define_binary_op!(Bsl);
define_binary_op!(Bsr);
define_binary_op!(Bxor);
define_binary_op!(Or);
define_binary_op!(Orelse);
define_binary_op!(Xor);
define_binary_op!(Match, Pattern);
define_binary_op!(DivFloat);
define_binary_op!(Div);
define_binary_op!(Rem);
define_binary_op!(Sub);
define_binary_op!(ListSub);
define_binary_op!(Add);
define_binary_op!(ListAdd);
define_binary_op!(Mul);
define_binary_op!(Eq);
define_binary_op!(ExactEq);
define_binary_op!(NotEq);
define_binary_op!(ExactNotEq);
define_binary_op!(Greater);
define_binary_op!(GreaterEq);
define_binary_op!(Less);
define_binary_op!(LessEq);
