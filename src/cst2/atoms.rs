use {Result, TokenReader, Parse, TokenRange, ErrorKind};

macro_rules! define_atom {
    ($name:ident, $($value:pat)|*) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            position: usize,
        }
        impl<'token, 'text: 'token> Parse<'token, 'text> for $name {
            fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
                let position = reader.position();
                let atom = track_try!(reader.read_atom());
                match atom.value() {
                    $($value)|* => Ok($name{ position }),
                    _ => track_panic!(ErrorKind::InvalidInput,
                                      "actual={:?}, expected={:?}",
                                      atom.value(),
                                      stringify!($($value)|*)),
                }
            }
        }
        impl TokenRange for $name {
            fn token_start(&self) -> usize {
                self.position
            }
            fn token_end(&self) -> usize {
                self.position + 1
            }
        }
    }
}

define_atom!(Module, "module");
define_atom!(Behaviour, "behaviour" | "behavior");
define_atom!(Export, "export");
define_atom!(ExportType, "export_type");
define_atom!(Import, "import");
define_atom!(Spec, "spec" | "callback"); // TODO
define_atom!(File, "file");
define_atom!(Record, "record");
define_atom!(Type, "type" | "opaque");
define_atom!(Define, "define");
define_atom!(Undef, "undef");
define_atom!(Ifdef, "ifdef");
define_atom!(Ifndef, "ifndef");
define_atom!(Else, "else");
define_atom!(Endif, "endif");
