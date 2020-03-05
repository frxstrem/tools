use std::fmt::Debug;
use std::marker::PhantomData;

pub fn parse<'a, T: Parse<'a>>(s: &'a str) -> Option<T> {
    let mut buf = ParseBuffer::new(s);
    let value = T::parse(&mut buf)?;
    if buf.is_empty() {
        Some(value)
    } else {
        None
    }
}

#[derive(Copy, Clone)]
pub struct ParseBuffer<'a>(&'a str);

impl<'a> ParseBuffer<'a> {
    pub fn new(data: &'a str) -> ParseBuffer<'a> {
        ParseBuffer(data)
    }

    pub fn parse<T: Parse<'a>>(&mut self) -> Option<T> {
        T::parse(self)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_next<T: Parse<'a>>(&self) -> bool {
        let mut buf = *self;
        T::parse(&mut buf).is_some()
    }
}

pub trait Parse<'a>: 'a + Debug + Sized {
    fn parse(buf: &mut ParseBuffer<'a>) -> Option<Self>;
}

impl<'a, T: Token<'a>> Parse<'a> for T {
    fn parse(buf: &mut ParseBuffer<'a>) -> Option<Self> {
        match T::parse_token(buf.0) {
            Some((value, next)) => {
                buf.0 = next;
                Some(value)
            }
            None => None,
        }
    }
}

macro_rules! tuple_parse {
    ($($ident:ident: $type:ident),* $(,)?) => {
        impl<'a, $($type: Parse<'a>),*> Parse<'a> for ($($type,)*) {
            fn parse(buf: &mut ParseBuffer<'a>) -> Option<($($type,)*)> {
                $(
                    let $ident = <$type as Parse<'a>>::parse(buf)?;
                )*
                Some(($($ident,)*))
            }
        }
    }
}

tuple_parse!(t: T);
tuple_parse!(t1: T1, t2: T2);
tuple_parse!(t1: T1, t2: T2, t3: T3);
tuple_parse!(t1: T1, t2: T2, t3: T3, t4: T4);
tuple_parse!(t1: T1, t2: T2, t3: T3, t4: T4, t5: T5);
tuple_parse!(t1: T1, t2: T2, t3: T3, t4: T4, t5: T5, t6: T6);
tuple_parse!(t1: T1, t2: T2, t3: T3, t4: T4, t5: T5, t6: T6, t7: T7);

pub trait Token<'a>: 'a + Debug + Sized {
    fn parse_token(s: &'a str) -> Option<(Self, &'a str)>;
}

#[derive(Debug)]
pub struct Punctuated<'a, T, P>
where
    T: Parse<'a>,
    P: Parse<'a>,
{
    items: Vec<(T, Option<P>)>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, T, P> Punctuated<'a, T, P>
where
    T: Parse<'a>,
    P: Parse<'a>,
{
    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|(item, _)| item)
    }
}

impl<'a, T, P> Parse<'a> for Punctuated<'a, T, P>
where
    T: Parse<'a>,
    P: Parse<'a>,
{
    fn parse(buf: &mut ParseBuffer<'a>) -> Option<Punctuated<'a, T, P>> {
        let mut items = Vec::new();
        while let Some(token) = buf.parse::<T>() {
            let punct = buf.parse::<P>();

            let no_punct = punct.is_none();
            items.push((token, punct));

            if no_punct {
                break;
            }
        }

        Some(Punctuated {
            items,
            _lifetime: PhantomData,
        })
    }
}
