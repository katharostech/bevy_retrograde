//! BDF Parser

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Parser Error: {0}")]
    ParserError(#[from] ParseError<LineCol>),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

pub fn parse<R: std::io::Read>(mut reader: R) -> Result<Font, Error> {
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    Ok(font_parser::font(&contents)?)
}

pub use ast::*;
use peg::{error::ParseError, str::LineCol};
mod ast {
    use bevy::utils::HashMap;

    #[derive(Debug, Clone)]
    pub struct Font {
        pub font_spec: String,
        pub font_size: (u32, u32, u32),
        pub glyphs: HashMap<char, Glyph>,
        pub bounds: BoundingBox,
        pub comments: Vec<String>,
        pub properties: HashMap<String, Property>,
    }

    #[derive(Debug, Clone)]
    pub enum Property {
        String(String),
        Int(i32),
    }

    #[derive(Debug, Clone)]
    pub struct BoundingBox {
        pub width: u32,
        pub height: u32,
        pub x: i32,
        pub y: i32,
    }

    #[derive(Debug, Clone)]
    pub struct Glyph {
        pub codepoint: char,
        pub device_width: (u32, u32),
        pub scalable_width: (u32, u32),
        pub bounds: BoundingBox,
        pub bitmap: Bitmap,
    }

    #[derive(Debug, Clone)]
    pub struct Bitmap {
        width: u32,
        height: u32,
        bits: Vec<bool>,
    }

    impl Bitmap {
        pub fn new(width: u32, height: u32) -> Self {
            Self {
                width,
                height,
                bits: vec![false; (width * height) as usize],
            }
        }

        fn x_y_to_i(&self, x: u32, y: u32) -> usize {
            (y * self.width + x) as usize
        }

        pub fn get(&self, x: u32, y: u32) -> bool {
            self.bits[self.x_y_to_i(x, y)]
        }

        pub fn set(&mut self, x: u32, y: u32, value: bool) {
            let i = self.x_y_to_i(x, y);
            self.bits[i] = value;
        }
    }
}

peg::parser! {
  grammar font_parser() for str {
    pub rule font() -> Font
        =
        __
        "STARTFONT" _ supported_version() _ "\n"
        "FONT" _ font_spec:$((!"\n" [_])*) _ "\n"
        "SIZE" _ s1:uint() _ s2:uint() _ s3:uint() _ "\n"
        "FONTBOUNDINGBOX" _ bwidth:uint() _ bheight:uint() _ bx:int() _ by:int() _ "\n"
        comments:(
            // Qutoted string
            "COMMENT" _ "\"" s:$((!"\"" [_])*) "\"" _ "\n" { s } /
            // Unquoted string
            "COMMENT" _ s:$((!"\n" [_])*) "\n" { s }
        )*
        "STARTPROPERTIES" _ property_count:uint() "\n"
        properties:(
            !"ENDPROPERTIES" name:$(['A'..='Z'|'_']*) _ value:property_value() { (name.to_owned(), value) }
        )+
        "ENDPROPERTIES" _ "\n"
        "CHARS" _ char_count:uint() _ "\n"
        glyphs:(
            "STARTCHAR" _ name:$((!"\n" [_])*) "\n"
            "ENCODING" _ encoding:$("-"? ['0'..='9']+) _ "\n"
            "SWIDTH" _ s1:uint() _ s2:uint() _ "\n"
            "DWIDTH" _ d1:uint() _ d2:uint() _ "\n"
            "BBX" _ bw:uint() _ bh:uint() _ bx:int() _ by:int() _ "\n"
            "BITMAP" _ "\n"
            bitmap_data:(
                s:$(['0'..='9'|'A'..='F']+) _ "\n" {?
                    let e = "could not parse hex string in character bitmap definition";
                    if s.len() > 8 {
                        return Err(e);
                    }

                    let s = format!("{}{}", s, "0".repeat(8 - s.len()));

                    u32::from_str_radix(
                        &s,
                        16
                    )
                    .map_err(|_| e)
                }
            )*
            "ENDCHAR" _ "\n" {?
                // Skip chars with the encoding -1
                if encoding == "-1" {
                    return Ok(None);
                }

                let encoding: u32 = encoding
                    .parse()
                    .map_err(|_| "Could not parse character ENCODING as int")?;
                let codepoint = char::from_u32(encoding).ok_or("Could not parse character encoding")?;

                Ok(Some((
                    codepoint,
                    Glyph {
                        codepoint,
                        device_width: (d1, d2),
                        scalable_width: (s1, s2),
                        bounds: BoundingBox {
                            width: bw,
                            height: bh,
                            x: bx,
                            y: by,
                        },
                        bitmap: {
                            let mut b = Bitmap::new(bw, bh);

                            for (i, row_data) in bitmap_data.iter().enumerate() {
                                for x in 0..bw {
                                    let bit = (31 - x);
                                    let mask = 1 << bit;
                                    b.set(
                                        x,
                                        i as u32,
                                        mask & row_data == mask,
                                    );
                                }
                            }

                            b
                        }
                    }
                )))
            }
        )*
        "ENDFONT" __
        {
            Font {
                font_spec: font_spec.into(),
                font_size: (s1, s2, s3),
                glyphs: glyphs.into_iter().flatten().collect(),
                bounds: BoundingBox {
                    width: bwidth,
                    height: bheight,
                    x: bx,
                    y: by,
                },
                properties: properties.into_iter().collect(),
                comments: comments.into_iter().map(|x| x.to_owned()).collect(),
            }
        }

    rule supported_version() = quiet!{"2.1"} /
        expected!("Unsupported font file version. Supported versions: 2.1")

    rule property_value() -> Property =
        // Int
        i:int() _ "\n" { Property::Int(i) } /
        // Quoted string
        "\"" s:$((!"\"" [_])*) "\"" _ "\n" { Property::String(s.into()) } /
        // Unquoted string
        s:$((!"\n" [_])*) "\n" { Property::String(s.into()) }

    rule int() -> i32 = number:$("-"? ['0'..='9']+) {?
        number.parse().map_err(|e| "Could not parse integer")
    }
    rule uint() -> u32 = number:$(['0'..='9']+) {?
        number.parse().map_err(|e| "Could not parse integer")
    }

    rule _() = quiet! { [' '|'\t']* }
    rule __() = quiet! { [' '|'\t'|'\n']* }
  }
}
