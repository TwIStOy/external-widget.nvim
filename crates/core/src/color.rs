use std::fmt::Display;

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    fn from_u32(v: u32) -> Self {
        if v <= 0xffffff {
            // 24-bit, no alpha in this
            Self {
                r: ((v >> 16) & 0xff) as u8,
                g: ((v >> 8) & 0xff) as u8,
                b: (v & 0xff) as u8,
                a: 0xff,
            }
        } else {
            Self {
                r: ((v >> 24) & 0xff) as u8,
                g: ((v >> 16) & 0xff) as u8,
                b: ((v >> 8) & 0xff) as u8,
                a: (v & 0xff) as u8,
            }
        }
    }

    // fn from_value(v: rmpv::Value) -> anyhow::Result<Self> {}
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl TryFrom<rmpv::Value> for Color {
    type Error = anyhow::Error;

    fn try_from(value: rmpv::Value) -> Result<Self, Self::Error> {
        if let Some(v) = value.as_u64() {
            Ok(Color::from_u32(v as u32))
        } else if let Some(v) = value.as_str() {
            let r = u8::from_str_radix(&v[0..2], 16)?;
            let g = u8::from_str_radix(&v[2..4], 16)?;
            let b = u8::from_str_radix(&v[4..6], 16)?;
            let a = if v.len() == 6 {
                // 24-bit, no alpha in this
                0xff
            } else {
                u8::from_str_radix(&v[6..8], 16)?
            };
            Ok(Color { r, g, b, a })
        } else {
            anyhow::bail!("invalid color")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_string() {
        let c = super::Color::new(0x12, 0x34, 0x56, 0x78);
        assert_eq!(c.to_string(), "#12345678");
        let c = super::Color::new(0x12, 0x34, 0x56, 0xff);
        assert_eq!(c.to_string(), "#123456ff");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x00);
        assert_eq!(c.to_string(), "#12345600");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x01);
        assert_eq!(c.to_string(), "#12345601");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x02);
        assert_eq!(c.to_string(), "#12345602");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x03);
        assert_eq!(c.to_string(), "#12345603");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x04);
        assert_eq!(c.to_string(), "#12345604");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x05);
        assert_eq!(c.to_string(), "#12345605");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x06);
        assert_eq!(c.to_string(), "#12345606");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x07);
        assert_eq!(c.to_string(), "#12345607");
        let c = super::Color::new(0x12, 0x34, 0x56, 0x08);
        assert_eq!(c.to_string(), "#12345608");
    }

    #[test]
    fn test_color_from_24bit() {
        let value = rmpv::Value::from(0x123456);
        let color: Color = value.try_into().unwrap();
        assert_eq!(color.to_string(), "#123456ff");
    }

    #[test]
    fn test_color_from_32bit() {
        let value = rmpv::Value::from(0x12345678);
        let color: Color = value.try_into().unwrap();
        assert_eq!(color.to_string(), "#12345678");
    }

    #[test]
    fn test_color_from_6str() {
        let value = rmpv::Value::from("123456");
        let color: Color = value.try_into().unwrap();
        assert_eq!(color.to_string(), "#123456ff");
    }

    #[test]
    fn test_color_from_8str() {
        let value = rmpv::Value::from("12345678");
        let color: Color = value.try_into().unwrap();
        assert_eq!(color.to_string(), "#12345678");
    }
}
